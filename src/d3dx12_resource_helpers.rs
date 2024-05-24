use core::num;
use std::{
    alloc::{alloc, Layout},
    ffi::c_void,
    mem::{self, size_of, MaybeUninit},
    ptr::{self, copy_nonoverlapping},
};

//*********************************************************
//
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License (MIT).
//
//*********************************************************
use windows::Win32::Graphics::Direct3D12::*;

use crate::d3dx12_core::CD3DX12_TEXTURE_COPY_LOCATION;

// #pragma once

// #ifndef __cplusplus
// #error D3DX12 requires C++
// #endif

// #include "d3d12.h"
// #include "d3dx12_core.h"
// #include "d3dx12_property_format_table.h"
// //------------------------------------------------------------------------------------------------
// template <typename T, typename U, typename V>
// inline void D3D12DecomposeSubresource( UINT Subresource, UINT MipLevels, UINT ArraySize, _Out_ T& MipSlice, _Out_ U& ArraySlice, _Out_ V& PlaneSlice ) noexcept
// {
//     MipSlice = static_cast<T>(Subresource % MipLevels);
//     ArraySlice = static_cast<U>((Subresource / MipLevels) % ArraySize);
//     PlaneSlice = static_cast<V>(Subresource / (MipLevels * ArraySize));
// }

//------------------------------------------------------------------------------------------------
// Row-by-row memcpy
#[inline]
unsafe fn MemcpySubresource(
    pDest: &D3D12_MEMCPY_DEST,
    pSrc: &D3D12_SUBRESOURCE_DATA,
    RowSizeInBytes: usize,
    NumRows: u32,
    NumSlices: u32,
) {
    for z in 0..NumSlices as usize {
        let pDestSlice = (pDest.pData as *mut u8).add(pDest.SlicePitch * z);
        let pSrcSlice = (pSrc.pData as *mut u8).add(pSrc.SlicePitch as usize * z);
        for y in 0..NumRows as usize {
            copy_nonoverlapping(
                pSrcSlice.add(pSrc.RowPitch as usize * y),
                pDestSlice.add(pDest.RowPitch as usize * y),
                RowSizeInBytes,
            );
        }
    }
}

//------------------------------------------------------------------------------------------------
// Row-by-row memcpy
#[inline]
unsafe fn memcpy_subresource_with_resource_data(
    p_dest: &D3D12_MEMCPY_DEST,
    p_resource_data: *const c_void,
    p_src: &D3D12_SUBRESOURCE_INFO,
    row_size_in_bytes: usize,
    num_rows: u32,
    num_slices: u32,
) {
    for z in 0..num_slices as usize {
        let pDestSlice = (p_dest.pData as *mut u8).add(p_dest.SlicePitch * z);
        let pSrcSlice = (p_resource_data as *mut u8)
            .add(p_src.Offset as usize)
            .add(p_src.DepthPitch as usize * z);
        for y in 0..num_rows as usize {
            copy_nonoverlapping(
                pSrcSlice.add(p_src.RowPitch as usize * y),
                pDestSlice.add(p_dest.RowPitch as usize * y),
                row_size_in_bytes,
            );
        }
    }
}

// //------------------------------------------------------------------------------------------------
// // Returns required size of a buffer to be used for data upload
#[inline]
pub fn get_required_intermediate_size(
    destination_resource: &ID3D12Resource,
    first_subresource: u32,
    num_subresources: u32,
) -> u64 {
    debug_assert!(first_subresource <= D3D12_REQ_SUBRESOURCES);
    debug_assert!(num_subresources <= D3D12_REQ_SUBRESOURCES - first_subresource);

    // #if defined(_MSC_VER) || !defined(_WIN32)
    //     const auto Desc = pDestinationResource->GetDesc();
    // #else
    //     D3D12_RESOURCE_DESC tmpDesc;
    //     const auto& Desc = *pDestinationResource->GetDesc(&tmpDesc);
    // #endif
    // D3D12_RESOURCE_DESC tmpDesc;
    let desc = unsafe { destination_resource.GetDesc() };
    let mut required_size = 0;

    let mut device: Option<ID3D12Device> = None;
    if let Ok(_) = unsafe { destination_resource.GetDevice(&mut device) } {
        if let Some(device) = device {
            unsafe {
                device.GetCopyableFootprints(
                    &desc,
                    first_subresource,
                    num_subresources,
                    0,
                    None,
                    None,
                    None,
                    Some(&mut required_size),
                );
            }
        };
    };
    // pDevice->Release();

    return required_size;
}
// inline UINT64 GetRequiredIntermediateSize(
//     _In_ ID3D12Resource* pDestinationResource,
//     _In_range_(0,D3D12_REQ_SUBRESOURCES) UINT FirstSubresource,
//     _In_range_(0,D3D12_REQ_SUBRESOURCES-FirstSubresource) UINT NumSubresources) noexcept
// {
// #if defined(_MSC_VER) || !defined(_WIN32)
//     const auto Desc = pDestinationResource->GetDesc();
// #else
//     D3D12_RESOURCE_DESC tmpDesc;
//     const auto& Desc = *pDestinationResource->GetDesc(&tmpDesc);
// #endif
//     UINT64 RequiredSize = 0;

//     ID3D12Device* pDevice = nullptr;
//     pDestinationResource->GetDevice(IID_ID3D12Device, reinterpret_cast<void**>(&pDevice));
//     pDevice->GetCopyableFootprints(&Desc, FirstSubresource, NumSubresources, 0, nullptr, nullptr, nullptr, &RequiredSize);
//     pDevice->Release();

//     return RequiredSize;
// }

//------------------------------------------------------------------------------------------------
// All arrays must be populated (e.g. by calling GetCopyableFootprints)
#[inline]
pub fn update_subresources(
    p_cmd_list: &ID3D12GraphicsCommandList,
    p_destination_resource: &ID3D12Resource,
    p_intermediate: &ID3D12Resource,
    first_subresource: u32,
    num_subresources: u32,
    required_size: u64,
    p_layouts: &[D3D12_PLACED_SUBRESOURCE_FOOTPRINT],
    p_num_rows: &[u32],
    p_row_sizes_in_bytes: &[u64],
    p_src_data: &[D3D12_SUBRESOURCE_DATA],
) -> u64 {
    assert!(first_subresource < D3D12_REQ_SUBRESOURCES);
    assert!(num_subresources < D3D12_REQ_SUBRESOURCES - first_subresource);
    // Minor validation
    // #if defined(_MSC_VER) || !defined(_WIN32)
    let intermediate_desc = unsafe { p_intermediate.GetDesc() };
    let destination_desc = unsafe { p_destination_resource.GetDesc() };
    // #else
    //     D3D12_RESOURCE_DESC tmpDesc1, tmpDesc2;
    //     const auto& IntermediateDesc = *pIntermediate->GetDesc(&tmpDesc1);
    //     const auto& DestinationDesc = *pDestinationResource->GetDesc(&tmpDesc2);
    // #endif
    if intermediate_desc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER
        || intermediate_desc.Width < required_size + p_layouts[0].Offset
        || required_size as usize > usize::MAX
        || (destination_desc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER
            && (first_subresource != 0 || num_subresources != 1))
    {
        return 0;
    }

    let mut pData: *mut c_void = ptr::null_mut();
    if let Err(_) = unsafe { p_intermediate.Map(0, None, Some(&mut pData)) } {
        return 0;
    }

    for i in 0..num_subresources as usize {
        if p_row_sizes_in_bytes[i] > usize::MAX as u64 {
            return 0;
        };
        let dest_data = D3D12_MEMCPY_DEST {
            pData: unsafe { pData.add(p_layouts[i].Offset as usize) } as _,
            RowPitch: p_layouts[i].Footprint.RowPitch as usize,
            SlicePitch: p_layouts[i].Footprint.RowPitch as usize * p_num_rows[i] as usize,
        };
        unsafe {
            MemcpySubresource(
                &dest_data,
                &p_src_data[i],
                p_row_sizes_in_bytes[i] as usize,
                p_num_rows[i],
                p_layouts[i].Footprint.Depth,
            )
        };
    }
    unsafe { p_intermediate.Unmap(0, None) };

    if destination_desc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER {
        unsafe {
            p_cmd_list.CopyBufferRegion(
                p_destination_resource,
                0,
                p_intermediate,
                p_layouts[0].Offset,
                p_layouts[0].Footprint.Width as u64,
            )
        };
    } else {
        for i in 0..num_subresources {
            let dst = CD3DX12_TEXTURE_COPY_LOCATION::new_with_subresource_index(
                p_destination_resource,
                i + first_subresource,
            );
            let src = CD3DX12_TEXTURE_COPY_LOCATION::new_with_placed_footprint(
                p_intermediate,
                &p_layouts[i as usize],
            );
            unsafe { p_cmd_list.CopyTextureRegion(&dst.0, 0, 0, 0, &src.0, None) };
        }
    }
    return required_size;
}

// //------------------------------------------------------------------------------------------------
// // All arrays must be populated (e.g. by calling GetCopyableFootprints)
// inline UINT64 UpdateSubresources(
//     _In_ ID3D12GraphicsCommandList* pCmdList,
//     _In_ ID3D12Resource* pDestinationResource,
//     _In_ ID3D12Resource* pIntermediate,
//     _In_range_(0,D3D12_REQ_SUBRESOURCES) UINT FirstSubresource,
//     _In_range_(0,D3D12_REQ_SUBRESOURCES-FirstSubresource) UINT NumSubresources,
//     UINT64 RequiredSize,
//     _In_reads_(NumSubresources) const D3D12_PLACED_SUBRESOURCE_FOOTPRINT* pLayouts,
//     _In_reads_(NumSubresources) const UINT* pNumRows,
//     _In_reads_(NumSubresources) const UINT64* pRowSizesInBytes,
//     _In_ const void* pResourceData,
//     _In_reads_(NumSubresources) const D3D12_SUBRESOURCE_INFO* pSrcData) noexcept
// {
//     // Minor validation
// #if defined(_MSC_VER) || !defined(_WIN32)
//     const auto IntermediateDesc = pIntermediate->GetDesc();
//     const auto DestinationDesc = pDestinationResource->GetDesc();
// #else
//     D3D12_RESOURCE_DESC tmpDesc1, tmpDesc2;
//     const auto& IntermediateDesc = *pIntermediate->GetDesc(&tmpDesc1);
//     const auto& DestinationDesc = *pDestinationResource->GetDesc(&tmpDesc2);
// #endif
//     if (IntermediateDesc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER ||
//         IntermediateDesc.Width < RequiredSize + pLayouts[0].Offset ||
//         RequiredSize > SIZE_T(-1) ||
//         (DestinationDesc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER &&
//             (FirstSubresource != 0 || NumSubresources != 1)))
//     {
//         return 0;
//     }

//     BYTE* pData;
//     HRESULT hr = pIntermediate->Map(0, nullptr, reinterpret_cast<void**>(&pData));
//     if (FAILED(hr))
//     {
//         return 0;
//     }

//     for (UINT i = 0; i < NumSubresources; ++i)
//     {
//         if (pRowSizesInBytes[i] > SIZE_T(-1)) return 0;
//         D3D12_MEMCPY_DEST DestData = { pData + pLayouts[i].Offset, pLayouts[i].Footprint.RowPitch, SIZE_T(pLayouts[i].Footprint.RowPitch) * SIZE_T(pNumRows[i]) };
//         MemcpySubresource(&DestData, pResourceData, &pSrcData[i], static_cast<SIZE_T>(pRowSizesInBytes[i]), pNumRows[i], pLayouts[i].Footprint.Depth);
//     }
//     pIntermediate->Unmap(0, nullptr);

//     if (DestinationDesc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER)
//     {
//         pCmdList->CopyBufferRegion(
//             pDestinationResource, 0, pIntermediate, pLayouts[0].Offset, pLayouts[0].Footprint.Width);
//     }
//     else
//     {
//         for (UINT i = 0; i < NumSubresources; ++i)
//         {
//             const CD3DX12_TEXTURE_COPY_LOCATION Dst(pDestinationResource, i + FirstSubresource);
//             const CD3DX12_TEXTURE_COPY_LOCATION Src(pIntermediate, pLayouts[i]);
//             pCmdList->CopyTextureRegion(&Dst, 0, 0, 0, &Src, nullptr);
//         }
//     }
//     return RequiredSize;
// }

// //------------------------------------------------------------------------------------------------
// // Heap-allocating UpdateSubresources implementation
// Heap-allocating UpdateSubresources implementation
pub fn update_subresources_heap(
    cmd_list: &ID3D12GraphicsCommandList,
    destination_resource: &ID3D12Resource,
    intermediate_resource: &ID3D12Resource,
    intermediate_offset: u64,
    first_subresource: u32,
    num_subresources: u32,
    src_data: &[D3D12_SUBRESOURCE_DATA],
) -> u64 {
    let mut layouts =
        Vec::<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>::with_capacity(num_subresources as usize);
    let mut num_rows = Vec::<u32>::with_capacity(num_subresources as usize);
    let mut row_size_in_bytes = Vec::<u64>::with_capacity(num_subresources as usize);

    let desc = unsafe { destination_resource.GetDesc() };

    let mut required_size = MaybeUninit::<u64>::uninit();
    unsafe {
        let device = {
            let mut result__: Option<ID3D12Device> = None;
            let Ok(()) = destination_resource.GetDevice(&mut result__) else {
                return 0;
            };
            result__.expect("ID3D12Device is null")
        };
        device.GetCopyableFootprints(
            &desc,
            first_subresource,
            num_subresources,
            intermediate_offset,
            Some(layouts.as_mut_ptr()),
            Some(num_rows.as_mut_ptr()),
            Some(row_size_in_bytes.as_mut_ptr()),
            Some(required_size.as_mut_ptr()),
        );
        layouts.set_len(num_subresources as usize);
        num_rows.set_len(num_subresources as usize);
        row_size_in_bytes.set_len(num_subresources as usize);
    };
    update_subresources(
        cmd_list,
        destination_resource,
        intermediate_resource,
        first_subresource,
        num_subresources,
        unsafe { required_size.assume_init() },
        &layouts,
        &num_rows,
        &row_size_in_bytes,
        src_data,
    )
}
// #[inline]
// pub fn update_subresources_heap(
//     pCmdList: &ID3D12GraphicsCommandList,
//     pDestinationResource: &ID3D12Resource,
//     pIntermediate: &ID3D12Resource,
//     IntermediateOffset: u64,
//     FirstSubresource: u32,
//     NumSubresources: u32,
//     pSrcData: &[D3D12_SUBRESOURCE_DATA],
// ) -> u64 {
//     assert!(FirstSubresource <= D3D12_REQ_SUBRESOURCES);
//     assert!(NumSubresources <= D3D12_REQ_SUBRESOURCES - FirstSubresource);

//     let RequiredSize = 0;
//     let MemToAlloc =
//         (size_of::<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>() + size_of::<u32>() + size_of::<u64>())
//             * NumSubresources as usize;
//     if MemToAlloc > isize::MAX as usize {
//         return 0;
//     }
//     let Ok(layout) = Layout::from_size_align(MemToAlloc, 8) else {
//         return 0;
//     };
//     let ptr = unsafe { alloc(layout) };
//     if ptr.is_null() {
//         return 0;
//     }
// void* pMem = HeapAlloc(GetProcessHeap(), 0, static_cast<SIZE_T>(MemToAlloc));
// if (pMem == nullptr)
// {
//    return 0;
// }

//         let pMem: Vec<D3D12_PLACED_SUBRESOURCE_FOOTPRINT> = Vec::with_capacity(NumSubresources as usize);

//     auto pLayouts = static_cast<D3D12_PLACED_SUBRESOURCE_FOOTPRINT*>(pMem);
//     auto pRowSizesInBytes = reinterpret_cast<UINT64*>(pLayouts + NumSubresources);
//     auto pNumRows = reinterpret_cast<UINT*>(pRowSizesInBytes + NumSubresources);

// #if defined(_MSC_VER) || !defined(_WIN32)
//     const auto Desc = pDestinationResource->GetDesc();
// #else
//     D3D12_RESOURCE_DESC tmpDesc;
//     const auto& Desc = *pDestinationResource->GetDesc(&tmpDesc);
// #endif
//     ID3D12Device* pDevice = nullptr;
//     pDestinationResource->GetDevice(IID_ID3D12Device, reinterpret_cast<void**>(&pDevice));
//     pDevice->GetCopyableFootprints(&Desc, FirstSubresource, NumSubresources, IntermediateOffset, pLayouts, pNumRows, pRowSizesInBytes, &RequiredSize);
//     pDevice->Release();

//     const UINT64 Result = UpdateSubresources(pCmdList, pDestinationResource, pIntermediate, FirstSubresource, NumSubresources, RequiredSize, pLayouts, pNumRows, pRowSizesInBytes, pSrcData);
//     HeapFree(GetProcessHeap(), 0, pMem);
//     return Result;
//     0
// }

// //------------------------------------------------------------------------------------------------
// // Heap-allocating UpdateSubresources implementation
// inline UINT64 UpdateSubresources(
//     _In_ ID3D12GraphicsCommandList* pCmdList,
//     _In_ ID3D12Resource* pDestinationResource,
//     _In_ ID3D12Resource* pIntermediate,
//     UINT64 IntermediateOffset,
//     _In_range_(0,D3D12_REQ_SUBRESOURCES) UINT FirstSubresource,
//     _In_range_(0,D3D12_REQ_SUBRESOURCES-FirstSubresource) UINT NumSubresources,
//     _In_ const void* pResourceData,
//     _In_reads_(NumSubresources) const D3D12_SUBRESOURCE_INFO* pSrcData) noexcept
// {
//     UINT64 RequiredSize = 0;
//     const auto MemToAlloc = static_cast<UINT64>(sizeof(D3D12_PLACED_SUBRESOURCE_FOOTPRINT) + sizeof(UINT) + sizeof(UINT64)) * NumSubresources;
//     if (MemToAlloc > SIZE_MAX)
//     {
//         return 0;
//     }
//     void* pMem = HeapAlloc(GetProcessHeap(), 0, static_cast<SIZE_T>(MemToAlloc));
//     if (pMem == nullptr)
//     {
//         return 0;
//     }
//     auto pLayouts = static_cast<D3D12_PLACED_SUBRESOURCE_FOOTPRINT*>(pMem);
//     auto pRowSizesInBytes = reinterpret_cast<UINT64*>(pLayouts + NumSubresources);
//     auto pNumRows = reinterpret_cast<UINT*>(pRowSizesInBytes + NumSubresources);

// #if defined(_MSC_VER) || !defined(_WIN32)
//     const auto Desc = pDestinationResource->GetDesc();
// #else
//     D3D12_RESOURCE_DESC tmpDesc;
//     const auto& Desc = *pDestinationResource->GetDesc(&tmpDesc);
// #endif
//     ID3D12Device* pDevice = nullptr;
//     pDestinationResource->GetDevice(IID_ID3D12Device, reinterpret_cast<void**>(&pDevice));
//     pDevice->GetCopyableFootprints(&Desc, FirstSubresource, NumSubresources, IntermediateOffset, pLayouts, pNumRows, pRowSizesInBytes, &RequiredSize);
//     pDevice->Release();

//     const UINT64 Result = UpdateSubresources(pCmdList, pDestinationResource, pIntermediate, FirstSubresource, NumSubresources, RequiredSize, pLayouts, pNumRows, pRowSizesInBytes, pResourceData, pSrcData);
//     HeapFree(GetProcessHeap(), 0, pMem);
//     return Result;
// }

//------------------------------------------------------------------------------------------------
// Stack-allocating UpdateSubresources implementation
// template <UINT MaxSubresources>
#[inline]
pub fn update_subresources_stack<const MAX_SUBRESOURCES: usize>(
    p_cmd_list: &ID3D12GraphicsCommandList,
    p_destination_resource: &ID3D12Resource,
    p_intermediate: &ID3D12Resource,
    intermediate_offset: u64,
    first_subresource: u32,
    num_subresources: u32,
    p_src_data: &[D3D12_SUBRESOURCE_DATA],
) -> u64 {
    let mut RequiredSize: u64 = 0;
    let mut layouts_backing: [MaybeUninit<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>; MAX_SUBRESOURCES] =
        unsafe { MaybeUninit::uninit().assume_init() };
    let mut num_rows_backing: [MaybeUninit<u32>; MAX_SUBRESOURCES] =
        unsafe { MaybeUninit::uninit().assume_init() };
    let mut row_sizes_in_bytes_backing: [MaybeUninit<u64>; MAX_SUBRESOURCES] =
        unsafe { MaybeUninit::uninit().assume_init() };

    // #if defined(_MSC_VER) || !defined(_WIN32)
    //     const auto Desc = pDestinationResource->GetDesc();
    // #else
    //     D3D12_RESOURCE_DESC tmpDesc;
    //     const auto& Desc = *pDestinationResource->GetDesc(&tmpDesc);
    // #endif

    let desc = unsafe { p_destination_resource.GetDesc() };

    let device = {
        let mut device: Option<ID3D12Device> = None;
        unsafe { p_destination_resource.GetDevice(&mut device) }
            .map(|()| device.unwrap())
            .unwrap()
    };

    unsafe {
        device.GetCopyableFootprints(
            &desc,
            first_subresource,
            num_subresources,
            intermediate_offset,
            Some(layouts_backing.as_mut_ptr() as _),
            Some(num_rows_backing.as_mut_ptr() as _),
            Some(row_sizes_in_bytes_backing.as_mut_ptr() as _),
            Some(&mut RequiredSize),
        );
    }

    let layouts = unsafe { mem::transmute(&layouts_backing[0..num_subresources as usize]) };
    let num_rows = unsafe { mem::transmute(&num_rows_backing[0..num_subresources as usize]) };
    let row_sizes_in_bytes =
        unsafe { mem::transmute(&row_sizes_in_bytes_backing[0..num_subresources as usize]) };
    return update_subresources(
        p_cmd_list,
        p_destination_resource,
        p_intermediate,
        first_subresource,
        num_subresources,
        RequiredSize,
        layouts,
        num_rows,
        row_sizes_in_bytes,
        p_src_data,
    );
}

// //------------------------------------------------------------------------------------------------
// // Stack-allocating UpdateSubresources implementation
// template <UINT MaxSubresources>
// inline UINT64 UpdateSubresources(
//     _In_ ID3D12GraphicsCommandList* pCmdList,
//     _In_ ID3D12Resource* pDestinationResource,
//     _In_ ID3D12Resource* pIntermediate,
//     UINT64 IntermediateOffset,
//     _In_range_(0,MaxSubresources) UINT FirstSubresource,
//     _In_range_(1,MaxSubresources-FirstSubresource) UINT NumSubresources,
//     _In_ const void* pResourceData,
//     _In_reads_(NumSubresources) const D3D12_SUBRESOURCE_INFO* pSrcData) noexcept
// {
//     UINT64 RequiredSize = 0;
//     D3D12_PLACED_SUBRESOURCE_FOOTPRINT Layouts[MaxSubresources];
//     UINT NumRows[MaxSubresources];
//     UINT64 RowSizesInBytes[MaxSubresources];

// #if defined(_MSC_VER) || !defined(_WIN32)
//     const auto Desc = pDestinationResource->GetDesc();
// #else
//     D3D12_RESOURCE_DESC tmpDesc;
//     const auto& Desc = *pDestinationResource->GetDesc(&tmpDesc);
// #endif
//     ID3D12Device* pDevice = nullptr;
//     pDestinationResource->GetDevice(IID_ID3D12Device, reinterpret_cast<void**>(&pDevice));
//     pDevice->GetCopyableFootprints(&Desc, FirstSubresource, NumSubresources, IntermediateOffset, Layouts, NumRows, RowSizesInBytes, &RequiredSize);
//     pDevice->Release();

//     return UpdateSubresources(pCmdList, pDestinationResource, pIntermediate, FirstSubresource, NumSubresources, RequiredSize, Layouts, NumRows, RowSizesInBytes, pResourceData, pSrcData);
// }

// //------------------------------------------------------------------------------------------------
// constexpr bool D3D12IsLayoutOpaque( D3D12_TEXTURE_LAYOUT Layout ) noexcept
// { return Layout == D3D12_TEXTURE_LAYOUT_UNKNOWN || Layout == D3D12_TEXTURE_LAYOUT_64KB_UNDEFINED_SWIZZLE; }

// //------------------------------------------------------------------------------------------------
// template< typename T >
// inline T D3DX12Align(T uValue, T uAlign)
// {
//     // Assert power of 2 alignment
//     D3DX12_ASSERT(0 == (uAlign & (uAlign - 1)));
//     T uMask = uAlign - 1;
//     T uResult = (uValue + uMask) & ~uMask;
//     D3DX12_ASSERT(uResult >= uValue);
//     D3DX12_ASSERT(0 == (uResult % uAlign));
//     return uResult;
// }

// //------------------------------------------------------------------------------------------------
// template< typename T >
// inline T D3DX12AlignAtLeast(T uValue, T uAlign)
// {
//     T aligned = D3DX12Align(uValue, uAlign);
//     return aligned > uAlign ? aligned : uAlign;
// }

// inline const CD3DX12_RESOURCE_DESC1* D3DX12ConditionallyExpandAPIDesc(
//     D3D12_RESOURCE_DESC1& LclDesc,
//     const D3D12_RESOURCE_DESC1* pDesc)
// {
//     return D3DX12ConditionallyExpandAPIDesc(static_cast<CD3DX12_RESOURCE_DESC1&>(LclDesc), static_cast<const CD3DX12_RESOURCE_DESC1*>(pDesc));
// }

// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 606)
// //------------------------------------------------------------------------------------------------
// // The difference between D3DX12GetCopyableFootprints and ID3D12Device::GetCopyableFootprints
// // is that this one loses a lot of error checking by assuming the arguments are correct
// inline bool D3DX12GetCopyableFootprints(
//     _In_  const D3D12_RESOURCE_DESC1& ResourceDesc,
//     _In_range_(0, D3D12_REQ_SUBRESOURCES) UINT FirstSubresource,
//     _In_range_(0, D3D12_REQ_SUBRESOURCES - FirstSubresource) UINT NumSubresources,
//     UINT64 BaseOffset,
//     _Out_writes_opt_(NumSubresources) D3D12_PLACED_SUBRESOURCE_FOOTPRINT* pLayouts,
//     _Out_writes_opt_(NumSubresources) UINT* pNumRows,
//     _Out_writes_opt_(NumSubresources) UINT64* pRowSizeInBytes,
//     _Out_opt_ UINT64* pTotalBytes)
// {
//     constexpr UINT64 uint64_max = ~0ull;
//     UINT64 TotalBytes = uint64_max;
//     UINT uSubRes = 0;

//     bool bResourceOverflow = false;
//     TotalBytes = 0;

//     const DXGI_FORMAT Format = ResourceDesc.Format;

//     CD3DX12_RESOURCE_DESC1 LresourceDesc;
//     const CD3DX12_RESOURCE_DESC1& resourceDesc = *D3DX12ConditionallyExpandAPIDesc(LresourceDesc, &ResourceDesc);

//     // Check if its a valid format
//     D3DX12_ASSERT(D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::FormatExists(Format));

//     const UINT WidthAlignment = D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::GetWidthAlignment( Format );
//     const UINT HeightAlignment = D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::GetHeightAlignment( Format );
//     const UINT16 DepthAlignment = UINT16( D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::GetDepthAlignment( Format ) );

//     for (; uSubRes < NumSubresources; ++uSubRes)
//     {
//         bool bOverflow = false;
//         UINT Subresource = FirstSubresource + uSubRes;

//         D3DX12_ASSERT(resourceDesc.MipLevels != 0);
//         UINT subresourceCount = resourceDesc.MipLevels * resourceDesc.ArraySize() * D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::GetPlaneCount(resourceDesc.Format);

//         if (Subresource > subresourceCount)
//         {
//             break;
//         }

//         TotalBytes = D3DX12Align< UINT64 >( TotalBytes, D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT );

//         UINT MipLevel, ArraySlice, PlaneSlice;
//         D3D12DecomposeSubresource(Subresource, resourceDesc.MipLevels, resourceDesc.ArraySize(), /*_Out_*/MipLevel, /*_Out_*/ArraySlice, /*_Out_*/PlaneSlice);

//         const UINT64 Width = D3DX12AlignAtLeast<UINT64>(resourceDesc.Width >> MipLevel, WidthAlignment);
//         const UINT Height =  D3DX12AlignAtLeast(resourceDesc.Height >> MipLevel, HeightAlignment);
//         const UINT16 Depth = D3DX12AlignAtLeast<UINT16>(resourceDesc.Depth() >> MipLevel, DepthAlignment);

//         // Adjust for the current PlaneSlice.  Most formats have only one plane.
//         DXGI_FORMAT PlaneFormat;
//         UINT32 MinPlanePitchWidth, PlaneWidth, PlaneHeight;
//         D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::GetPlaneSubsampledSizeAndFormatForCopyableLayout(PlaneSlice, Format, (UINT)Width, Height, /*_Out_*/ PlaneFormat, /*_Out_*/ MinPlanePitchWidth, /* _Out_ */ PlaneWidth, /*_Out_*/ PlaneHeight);

//         D3D12_SUBRESOURCE_FOOTPRINT LocalPlacement;
//         auto& Placement = pLayouts ? pLayouts[uSubRes].Footprint : LocalPlacement;
//         Placement.Format = PlaneFormat;
//         Placement.Width = PlaneWidth;
//         Placement.Height = PlaneHeight;
//         Placement.Depth = Depth;

//         // Calculate row pitch
//         UINT MinPlaneRowPitch = 0;
//         D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::CalculateMinimumRowMajorRowPitch(PlaneFormat, MinPlanePitchWidth, MinPlaneRowPitch);

//         // Formats with more than one plane choose a larger pitch alignment to ensure that each plane begins on the row
//         // immediately following the previous plane while still adhering to subresource alignment restrictions.
//         static_assert(   D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT >= D3D12_TEXTURE_DATA_PITCH_ALIGNMENT
//                         && ((D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT % D3D12_TEXTURE_DATA_PITCH_ALIGNMENT) == 0),
//                         "D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT  must be >= and evenly divisible by D3D12_TEXTURE_DATA_PITCH_ALIGNMENT." );

//         Placement.RowPitch = D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::Planar(Format)
//             ? D3DX12Align< UINT >( MinPlaneRowPitch, D3D12_TEXTURE_DATA_PLACEMENT_ALIGNMENT )
//             : D3DX12Align< UINT >( MinPlaneRowPitch, D3D12_TEXTURE_DATA_PITCH_ALIGNMENT );

//         if (pRowSizeInBytes)
//         {
//             UINT PlaneRowSize = 0;
//             D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::CalculateMinimumRowMajorRowPitch(PlaneFormat, PlaneWidth, PlaneRowSize);

//             pRowSizeInBytes[uSubRes] = PlaneRowSize;
//         }

//         // Number of rows (accounting for block compression and additional planes)
//         UINT NumRows = 0;
//         if (D3D12_PROPERTY_LAYOUT_FORMAT_TABLE::Planar(Format))
//         {
//             NumRows = PlaneHeight;
//         }
//         else
//         {
//             D3DX12_ASSERT(Height % HeightAlignment == 0);
//             NumRows = Height / HeightAlignment;
//         }

//         if (pNumRows)
//         {
//             pNumRows[uSubRes] = NumRows;
//         }

//             // Offsetting
//             if (pLayouts)
//             {
//                 pLayouts[uSubRes].Offset = (bOverflow ? uint64_max : TotalBytes + BaseOffset);
//             }

//         const UINT16 NumSlices = Depth;
//         const UINT64 SubresourceSize = (NumRows * NumSlices - 1) * Placement.RowPitch + MinPlaneRowPitch;

//         // uint64 addition with overflow checking
//         TotalBytes = TotalBytes + SubresourceSize;
//         if(TotalBytes < SubresourceSize)
//         {
//             TotalBytes = uint64_max;
//         }
//         bResourceOverflow  = bResourceOverflow  || bOverflow;
//     }

//     // Overflow error
//     if (bResourceOverflow)
//     {
//         TotalBytes = uint64_max;
//     }

//     if (pLayouts)
//     {
//         memset( pLayouts + uSubRes, -1, sizeof( *pLayouts ) * (NumSubresources - uSubRes) );
//     }
//     if (pNumRows)
//     {
//         memset(pNumRows + uSubRes, -1, sizeof(*pNumRows) * (NumSubresources - uSubRes));
//     }
//     if (pRowSizeInBytes)
//     {
//         memset(pRowSizeInBytes + uSubRes, -1, sizeof(*pRowSizeInBytes) * (NumSubresources - uSubRes));
//     }
//     if (pTotalBytes)
//     {
//         *pTotalBytes = TotalBytes;
//     }
//     if(TotalBytes == uint64_max)
//     {
//         return false;
//     }
//     return true;
// }

// //------------------------------------------------------------------------------------------------
// inline D3D12_RESOURCE_DESC1 D3DX12ResourceDesc0ToDesc1(D3D12_RESOURCE_DESC const& desc0)
// {
//     D3D12_RESOURCE_DESC1       desc1;
//     desc1.Dimension          = desc0.Dimension;
//     desc1.Alignment          = desc0.Alignment;
//     desc1.Width              = desc0.Width;
//     desc1.Height             = desc0.Height;
//     desc1.DepthOrArraySize   = desc0.DepthOrArraySize;
//     desc1.MipLevels          = desc0.MipLevels;
//     desc1.Format             = desc0.Format;
//     desc1.SampleDesc.Count   = desc0.SampleDesc.Count;
//     desc1.SampleDesc.Quality = desc0.SampleDesc.Quality;
//     desc1.Layout             = desc0.Layout;
//     desc1.Flags              = desc0.Flags;
//     desc1.SamplerFeedbackMipRegion.Width = 0;
//     desc1.SamplerFeedbackMipRegion.Height = 0;
//     desc1.SamplerFeedbackMipRegion.Depth = 0;
//     return desc1;
// }

// //------------------------------------------------------------------------------------------------
// inline bool D3DX12GetCopyableFootprints(
// 	_In_  const D3D12_RESOURCE_DESC& pResourceDesc,
// 	_In_range_(0, D3D12_REQ_SUBRESOURCES) UINT FirstSubresource,
// 	_In_range_(0, D3D12_REQ_SUBRESOURCES - FirstSubresource) UINT NumSubresources,
// 	UINT64 BaseOffset,
// 	_Out_writes_opt_(NumSubresources) D3D12_PLACED_SUBRESOURCE_FOOTPRINT* pLayouts,
// 	_Out_writes_opt_(NumSubresources) UINT* pNumRows,
// 	_Out_writes_opt_(NumSubresources) UINT64* pRowSizeInBytes,
// 	_Out_opt_ UINT64* pTotalBytes)
// {
//     // From D3D12_RESOURCE_DESC to D3D12_RESOURCE_DESC1
//     D3D12_RESOURCE_DESC1 desc = D3DX12ResourceDesc0ToDesc1(pResourceDesc);
// 	return D3DX12GetCopyableFootprints(
// 		*static_cast<CD3DX12_RESOURCE_DESC1*>(&desc),// From D3D12_RESOURCE_DESC1 to CD3DX12_RESOURCE_DESC1
// 		FirstSubresource,
// 		NumSubresources,
// 		BaseOffset,
// 		pLayouts,
// 		pNumRows,
// 		pRowSizeInBytes,
// 		pTotalBytes);
// }

// #endif // D3D12_SDK_VERSION >= 606
