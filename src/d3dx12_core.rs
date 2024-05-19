//*********************************************************
//
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License (MIT).
//
//*********************************************************

use std::ffi::c_void;

use windows::Win32::{
    Foundation::{BOOL, FALSE, RECT, TRUE},
    Graphics::{
        Direct3D::ID3DBlob,
        Direct3D12::*,
        Dxgi::Common::{DXGI_FORMAT, DXGI_FORMAT_UNKNOWN, DXGI_SAMPLE_DESC},
    },
};

// #pragma once

// #ifndef __cplusplus
// #error D3DX12 requires C++
// #endif

// #include <string.h>
// #include "d3d12.h"
// #include "d3dx12_default.h"

// //------------------------------------------------------------------------------------------------
// #ifndef D3DX12_ASSERT
//   #ifdef assert
//     #define D3DX12_ASSERT(x) assert(x)
//   #else
//     #define D3DX12_ASSERT(x)
//   #endif
// #endif

// //------------------------------------------------------------------------------------------------
// template <typename t_CommandListType>
// inline ID3D12CommandList * const * CommandListCast(t_CommandListType * const * pp) noexcept
// {
//     // This cast is useful for passing strongly typed command list pointers into
//     // ExecuteCommandLists.
//     // This cast is valid as long as the const-ness is respected. D3D12 APIs do
//     // respect the const-ness of their arguments.
//     return reinterpret_cast<ID3D12CommandList * const *>(pp);
// }

// //------------------------------------------------------------------------------------------------
// inline bool operator==( const D3D12_VIEWPORT& l, const D3D12_VIEWPORT& r ) noexcept
// {
//     return l.TopLeftX == r.TopLeftX && l.TopLeftY == r.TopLeftY && l.Width == r.Width &&
//         l.Height == r.Height && l.MinDepth == r.MinDepth && l.MaxDepth == r.MaxDepth;
// }

// //------------------------------------------------------------------------------------------------
// inline bool operator!=( const D3D12_VIEWPORT& l, const D3D12_VIEWPORT& r ) noexcept
// { return !( l == r ); }

// //------------------------------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub type D3D12_RECT = RECT;
#[allow(non_camel_case_types)]
pub trait CD3DX12_RECT {
    #[allow(clippy::new_ret_no_self)]
    fn new(left: i32, top: i32, right: i32, bottom: i32) -> D3D12_RECT {
        D3D12_RECT {
            left,
            top,
            right,
            bottom,
        }
    }
}
impl CD3DX12_RECT for D3D12_RECT {}

// //------------------------------------------------------------------------------------------------
#[allow(non_camel_case_types)]
pub trait CD3DX12_VIEWPORT {
    #[allow(clippy::new_ret_no_self)]
    fn new(
        top_left_x: f32,
        top_left_y: f32,
        width: f32,
        height: f32,
        min_depth: Option<f32>,
        max_depth: Option<f32>,
    ) -> D3D12_VIEWPORT {
        let min_depth = min_depth.unwrap_or(D3D12_MIN_DEPTH);
        let max_depth = max_depth.unwrap_or(D3D12_MAX_DEPTH);
        D3D12_VIEWPORT {
            TopLeftX: top_left_x,
            TopLeftY: top_left_y,
            Width: width,
            Height: height,
            MinDepth: min_depth,
            MaxDepth: max_depth,
        }
    }
    fn from_resource(
        resource: &ID3D12Resource,
        mip_slice: Option<u32>,
        top_left_x: Option<f32>,
        top_left_y: Option<f32>,
        min_depth: Option<f32>,
        max_depth: Option<f32>,
    ) -> D3D12_VIEWPORT {
        let mip_slice = mip_slice.unwrap_or(0u32);
        let top_left_x = top_left_x.unwrap_or(0f32);
        let top_left_y = top_left_y.unwrap_or(0f32);
        let min_depth = min_depth.unwrap_or(D3D12_MIN_DEPTH);
        let max_depth = max_depth.unwrap_or(D3D12_MAX_DEPTH);
        // #if defined(_MSC_VER) || !defined(_WIN32)
        //         const auto Desc = pResource->GetDesc();
        // #else
        //         D3D12_RESOURCE_DESC tmpDesc;
        //         const auto& Desc = *pResource->GetDesc(&tmpDesc);
        // #endif
        let Desc = unsafe { resource.GetDesc() };
        let SubresourceWidth: u64 = Desc.Width >> mip_slice;
        let SubresourceHeight: u64 = Desc.Height as u64 >> mip_slice;
        let TopLeftX: f32;
        let TopLeftY: f32;
        let Width: f32;
        let Height: f32;
        match Desc.Dimension {
            D3D12_RESOURCE_DIMENSION_BUFFER => {
                TopLeftX = top_left_x;
                TopLeftY = 0.0f32;
                Width = Desc.Width as f32 - top_left_x;
                Height = 1.0f32;
            }
            D3D12_RESOURCE_DIMENSION_TEXTURE1D => {
                TopLeftX = top_left_x;
                TopLeftY = 0.0f32;
                Width = (if SubresourceWidth != 0 {
                    SubresourceWidth as f32
                } else {
                    1.0f32
                }) - top_left_x;
                Height = 1.0f32;
            }
            D3D12_RESOURCE_DIMENSION_TEXTURE2D => {
                TopLeftX = top_left_x;
                TopLeftY = top_left_y;
                Width = (if SubresourceWidth != 0 {
                    SubresourceWidth as f32
                } else {
                    1.0f32
                }) - top_left_x;
                Height = (if SubresourceHeight != 0 {
                    SubresourceHeight as f32
                } else {
                    1.0f32
                }) - top_left_y;
            }
            D3D12_RESOURCE_DIMENSION_TEXTURE3D => {
                TopLeftX = top_left_x;
                TopLeftY = top_left_y;
                Width = (if SubresourceWidth != 0 {
                    SubresourceWidth as f32
                } else {
                    1.0f32
                }) - top_left_x;
                Height = (if SubresourceHeight != 0 {
                    SubresourceHeight as f32
                } else {
                    1.0f32
                }) - top_left_y;
            }
            _ => {
                TopLeftX = top_left_x;
                TopLeftY = top_left_y;
                Width = (if SubresourceWidth != 0 {
                    SubresourceWidth as f32
                } else {
                    1.0f32
                }) - top_left_x;
                Height = (if SubresourceHeight != 0 {
                    SubresourceHeight as f32
                } else {
                    1.0f32
                }) - top_left_y;
            }
        }

        D3D12_VIEWPORT {
            TopLeftX,
            TopLeftY,
            Width,
            Height,
            MinDepth: min_depth,
            MaxDepth: max_depth,
        }
    }
}
impl CD3DX12_VIEWPORT for D3D12_VIEWPORT {}

// //------------------------------------------------------------------------------------------------
#[allow(non_camel_case_types)]
trait CD3DX12_BOX {
    fn from_left_right(left: i32, right: i32) -> D3D12_BOX {
        D3D12_BOX {
            left: left as u32,
            top: 0,
            front: 0,
            right: right as u32,
            bottom: 1,
            back: 1,
        }
    }
    fn from_left_top_right_bottom(left: i32, top: i32, right: i32, bottom: i32) -> D3D12_BOX {
        D3D12_BOX {
            left: left as u32,
            top: top as u32,
            front: 0,
            right: right as u32,
            bottom: bottom as u32,
            back: 1,
        }
    }
    #[allow(clippy::new_ret_no_self)]
    fn new(left: i32, top: i32, front: i32, right: i32, bottom: i32, back: i32) -> D3D12_BOX {
        D3D12_BOX {
            left: left as u32,
            top: top as u32,
            front: front as u32,
            right: right as u32,
            bottom: bottom as u32,
            back: back as u32,
        }
    }
}
impl CD3DX12_BOX for D3D12_BOX {}

// //------------------------------------------------------------------------------------------------
const DEFAULT_STENCIL_OP: D3D12_DEPTH_STENCILOP_DESC = D3D12_DEPTH_STENCILOP_DESC {
    StencilFailOp: D3D12_STENCIL_OP_KEEP,
    StencilDepthFailOp: D3D12_STENCIL_OP_KEEP,
    StencilPassOp: D3D12_STENCIL_OP_KEEP,
    StencilFunc: D3D12_COMPARISON_FUNC_ALWAYS,
};
// //------------------------------------------------------------------------------------------------
pub trait CD3DX12_DEPTH_STENCIL_DESC {
    fn default() -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: TRUE,
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_LESS,
            StencilEnable: FALSE,
            StencilReadMask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
            StencilWriteMask: D3D12_DEFAULT_STENCIL_WRITE_MASK as u8,
            FrontFace: DEFAULT_STENCIL_OP,
            BackFace: DEFAULT_STENCIL_OP,
        }
    }
    fn new(
        depth_enable: BOOL,
        depth_write_mask: D3D12_DEPTH_WRITE_MASK,
        depth_func: D3D12_COMPARISON_FUNC,
        stencil_enable: BOOL,
        stencil_read_mask: u8,
        stencil_write_mask: u8,
        front_stencil_fail_op: D3D12_STENCIL_OP,
        front_stencil_depth_fail_op: D3D12_STENCIL_OP,
        front_stencil_pass_op: D3D12_STENCIL_OP,
        front_stencil_func: D3D12_COMPARISON_FUNC,
        back_stencil_fail_op: D3D12_STENCIL_OP,
        back_stencil_depth_fail_op: D3D12_STENCIL_OP,
        back_stencil_pass_op: D3D12_STENCIL_OP,
        back_stencil_func: D3D12_COMPARISON_FUNC,
    ) -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: depth_enable,
            DepthWriteMask: depth_write_mask,
            DepthFunc: depth_func,
            StencilEnable: stencil_enable,
            StencilReadMask: stencil_read_mask,
            StencilWriteMask: stencil_write_mask,
            FrontFace: D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: front_stencil_fail_op,
                StencilDepthFailOp: front_stencil_depth_fail_op,
                StencilPassOp: front_stencil_pass_op,
                StencilFunc: front_stencil_func,
            },
            BackFace: D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: back_stencil_fail_op,
                StencilDepthFailOp: back_stencil_depth_fail_op,
                StencilPassOp: back_stencil_pass_op,
                StencilFunc: back_stencil_func,
            },
        }
    }
}
impl CD3DX12_DEPTH_STENCIL_DESC for D3D12_DEPTH_STENCIL_DESC {}

// //------------------------------------------------------------------------------------------------
trait CD3DX12_DEPTH_STENCIL_DESC1 {
    fn from(o: &D3D12_DEPTH_STENCIL_DESC) -> D3D12_DEPTH_STENCIL_DESC1 {
        D3D12_DEPTH_STENCIL_DESC1 {
            DepthEnable: o.DepthEnable,
            DepthWriteMask: o.DepthWriteMask,
            DepthFunc: o.DepthFunc,
            StencilEnable: o.StencilEnable,
            StencilReadMask: o.StencilReadMask,
            StencilWriteMask: o.StencilWriteMask,
            FrontFace: D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: o.FrontFace.StencilFailOp,
                StencilDepthFailOp: o.FrontFace.StencilDepthFailOp,
                StencilPassOp: o.FrontFace.StencilPassOp,
                StencilFunc: o.FrontFace.StencilFunc,
            },
            BackFace: D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: o.BackFace.StencilFailOp,
                StencilDepthFailOp: o.BackFace.StencilDepthFailOp,
                StencilPassOp: o.BackFace.StencilPassOp,
                StencilFunc: o.BackFace.StencilFunc,
            },
            DepthBoundsTestEnable: FALSE,
        }
    }
    fn default() -> D3D12_DEPTH_STENCIL_DESC1 {
        D3D12_DEPTH_STENCIL_DESC1 {
            DepthEnable: TRUE,
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_LESS,
            StencilEnable: FALSE,
            StencilReadMask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
            StencilWriteMask: D3D12_DEFAULT_STENCIL_WRITE_MASK as u8,
            FrontFace: DEFAULT_STENCIL_OP,
            BackFace: DEFAULT_STENCIL_OP,
            DepthBoundsTestEnable: FALSE,
        }
    }
    fn new(
        depth_enable: BOOL,
        depth_write_mask: D3D12_DEPTH_WRITE_MASK,
        depth_func: D3D12_COMPARISON_FUNC,
        stencil_enable: BOOL,
        stencil_read_mask: u8,
        stencil_write_mask: u8,
        front_stencil_fail_op: D3D12_STENCIL_OP,
        front_stencil_depth_fail_op: D3D12_STENCIL_OP,
        front_stencil_pass_op: D3D12_STENCIL_OP,
        front_stencil_func: D3D12_COMPARISON_FUNC,
        back_stencil_fail_op: D3D12_STENCIL_OP,
        back_stencil_depth_fail_op: D3D12_STENCIL_OP,
        back_stencil_pass_op: D3D12_STENCIL_OP,
        back_stencil_func: D3D12_COMPARISON_FUNC,
        depth_bounds_test_enable: BOOL,
    ) -> D3D12_DEPTH_STENCIL_DESC1 {
        D3D12_DEPTH_STENCIL_DESC1 {
            DepthEnable: depth_enable,
            DepthWriteMask: depth_write_mask,
            DepthFunc: depth_func,
            StencilEnable: stencil_enable,
            StencilReadMask: stencil_read_mask,
            StencilWriteMask: stencil_write_mask,
            FrontFace: D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: front_stencil_fail_op,
                StencilDepthFailOp: front_stencil_depth_fail_op,
                StencilPassOp: front_stencil_pass_op,
                StencilFunc: front_stencil_func,
            },
            BackFace: D3D12_DEPTH_STENCILOP_DESC {
                StencilFailOp: back_stencil_fail_op,
                StencilDepthFailOp: back_stencil_depth_fail_op,
                StencilPassOp: back_stencil_pass_op,
                StencilFunc: back_stencil_func,
            },
            DepthBoundsTestEnable: depth_bounds_test_enable,
        }
    }
}

// //------------------------------------------------------------------------------------------------
// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 606)
// struct CD3DX12_DEPTH_STENCIL_DESC2 : public D3D12_DEPTH_STENCIL_DESC2
// {
//     CD3DX12_DEPTH_STENCIL_DESC2() = default;
//     explicit CD3DX12_DEPTH_STENCIL_DESC2( const D3D12_DEPTH_STENCIL_DESC2& o ) noexcept :
//         D3D12_DEPTH_STENCIL_DESC2( o )
//     {}
//     explicit CD3DX12_DEPTH_STENCIL_DESC2( const D3D12_DEPTH_STENCIL_DESC1& o ) noexcept
//     {
//         DepthEnable                  = o.DepthEnable;
//         DepthWriteMask               = o.DepthWriteMask;
//         DepthFunc                    = o.DepthFunc;
//         StencilEnable                = o.StencilEnable;
//         FrontFace.StencilFailOp      = o.FrontFace.StencilFailOp;
//         FrontFace.StencilDepthFailOp = o.FrontFace.StencilDepthFailOp;
//         FrontFace.StencilPassOp      = o.FrontFace.StencilPassOp;
//         FrontFace.StencilFunc        = o.FrontFace.StencilFunc;
//         FrontFace.StencilReadMask    = o.StencilReadMask;
//         FrontFace.StencilWriteMask   = o.StencilWriteMask;

//         BackFace.StencilFailOp       = o.BackFace.StencilFailOp;
//         BackFace.StencilDepthFailOp  = o.BackFace.StencilDepthFailOp;
//         BackFace.StencilPassOp       = o.BackFace.StencilPassOp;
//         BackFace.StencilFunc         = o.BackFace.StencilFunc;
//         BackFace.StencilReadMask     = o.StencilReadMask;
//         BackFace.StencilWriteMask    = o.StencilWriteMask;
//         DepthBoundsTestEnable        = o.DepthBoundsTestEnable;
//     }
//     explicit CD3DX12_DEPTH_STENCIL_DESC2( const D3D12_DEPTH_STENCIL_DESC& o ) noexcept
//     {
//         DepthEnable                  = o.DepthEnable;
//         DepthWriteMask               = o.DepthWriteMask;
//         DepthFunc                    = o.DepthFunc;
//         StencilEnable                = o.StencilEnable;

//         FrontFace.StencilFailOp      = o.FrontFace.StencilFailOp;
//         FrontFace.StencilDepthFailOp = o.FrontFace.StencilDepthFailOp;
//         FrontFace.StencilPassOp      = o.FrontFace.StencilPassOp;
//         FrontFace.StencilFunc        = o.FrontFace.StencilFunc;
//         FrontFace.StencilReadMask    = o.StencilReadMask;
//         FrontFace.StencilWriteMask   = o.StencilWriteMask;

//         BackFace.StencilFailOp       = o.BackFace.StencilFailOp;
//         BackFace.StencilDepthFailOp  = o.BackFace.StencilDepthFailOp;
//         BackFace.StencilPassOp       = o.BackFace.StencilPassOp;
//         BackFace.StencilFunc         = o.BackFace.StencilFunc;
//         BackFace.StencilReadMask     = o.StencilReadMask;
//         BackFace.StencilWriteMask    = o.StencilWriteMask;

//         DepthBoundsTestEnable        = FALSE;
//     }
//     explicit CD3DX12_DEPTH_STENCIL_DESC2( CD3DX12_DEFAULT ) noexcept
//     {
//         DepthEnable = TRUE;
//         DepthWriteMask = D3D12_DEPTH_WRITE_MASK_ALL;
//         DepthFunc = D3D12_COMPARISON_FUNC_LESS;
//         StencilEnable = FALSE;
//         const D3D12_DEPTH_STENCILOP_DESC1 defaultStencilOp =
//         { D3D12_STENCIL_OP_KEEP, D3D12_STENCIL_OP_KEEP, D3D12_STENCIL_OP_KEEP, D3D12_COMPARISON_FUNC_ALWAYS, D3D12_DEFAULT_STENCIL_READ_MASK, D3D12_DEFAULT_STENCIL_WRITE_MASK };
//         FrontFace = defaultStencilOp;
//         BackFace = defaultStencilOp;
//         DepthBoundsTestEnable = FALSE;
//     }
//     explicit CD3DX12_DEPTH_STENCIL_DESC2(
//         BOOL depthEnable,
//         D3D12_DEPTH_WRITE_MASK depthWriteMask,
//         D3D12_COMPARISON_FUNC depthFunc,
//         BOOL stencilEnable,
//         D3D12_STENCIL_OP frontStencilFailOp,
//         D3D12_STENCIL_OP frontStencilDepthFailOp,
//         D3D12_STENCIL_OP frontStencilPassOp,
//         D3D12_COMPARISON_FUNC frontStencilFunc,
//         UINT8 frontStencilReadMask,
//         UINT8 frontStencilWriteMask,
//         D3D12_STENCIL_OP backStencilFailOp,
//         D3D12_STENCIL_OP backStencilDepthFailOp,
//         D3D12_STENCIL_OP backStencilPassOp,
//         D3D12_COMPARISON_FUNC backStencilFunc,
//         UINT8 backStencilReadMask,
//         UINT8 backStencilWriteMask,
//         BOOL depthBoundsTestEnable ) noexcept
//     {
//         DepthEnable = depthEnable;
//         DepthWriteMask = depthWriteMask;
//         DepthFunc = depthFunc;
//         StencilEnable = stencilEnable;

//         FrontFace.StencilFailOp = frontStencilFailOp;
//         FrontFace.StencilDepthFailOp = frontStencilDepthFailOp;
//         FrontFace.StencilPassOp = frontStencilPassOp;
//         FrontFace.StencilFunc = frontStencilFunc;
//         FrontFace.StencilReadMask = frontStencilReadMask;
//         FrontFace.StencilWriteMask = frontStencilWriteMask;

//         BackFace.StencilFailOp = backStencilFailOp;
//         BackFace.StencilDepthFailOp = backStencilDepthFailOp;
//         BackFace.StencilPassOp = backStencilPassOp;
//         BackFace.StencilFunc = backStencilFunc;
//         BackFace.StencilReadMask = backStencilReadMask;
//         BackFace.StencilWriteMask = backStencilWriteMask;

//         DepthBoundsTestEnable = depthBoundsTestEnable;
//     }

//     operator D3D12_DEPTH_STENCIL_DESC() const noexcept
//     {
//         D3D12_DEPTH_STENCIL_DESC D;
//         D.DepthEnable = DepthEnable;
//         D.DepthWriteMask = DepthWriteMask;
//         D.DepthFunc = DepthFunc;
//         D.StencilEnable = StencilEnable;
//         D.StencilReadMask = FrontFace.StencilReadMask;
//         D.StencilWriteMask = FrontFace.StencilWriteMask;
//         D.FrontFace.StencilFailOp = FrontFace.StencilFailOp;
//         D.FrontFace.StencilDepthFailOp = FrontFace.StencilDepthFailOp;
//         D.FrontFace.StencilPassOp = FrontFace.StencilPassOp;
//         D.FrontFace.StencilFunc = FrontFace.StencilFunc;
//         D.BackFace.StencilFailOp = BackFace.StencilFailOp;
//         D.BackFace.StencilDepthFailOp = BackFace.StencilDepthFailOp;
//         D.BackFace.StencilPassOp = BackFace.StencilPassOp;
//         D.BackFace.StencilFunc = BackFace.StencilFunc;
//         return D;
//     }
// };
// #endif // D3D12_SDK_VERSION >= 606

// //------------------------------------------------------------------------------------------------
pub trait CD3DX12_BLEND_DESC {
    fn default() -> D3D12_BLEND_DESC {
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [D3D12_RENDER_TARGET_BLEND_DESC {
                BlendEnable: FALSE,
                LogicOpEnable: FALSE,
                SrcBlend: D3D12_BLEND_ONE,
                DestBlend: D3D12_BLEND_ZERO,
                BlendOp: D3D12_BLEND_OP_ADD,
                SrcBlendAlpha: D3D12_BLEND_ONE,
                DestBlendAlpha: D3D12_BLEND_ZERO,
                BlendOpAlpha: D3D12_BLEND_OP_ADD,
                LogicOp: D3D12_LOGIC_OP_NOOP,
                RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
            }; D3D12_SIMULTANEOUS_RENDER_TARGET_COUNT as usize],
        }
    }
}
impl CD3DX12_BLEND_DESC for D3D12_BLEND_DESC {}

// //------------------------------------------------------------------------------------------------
pub trait CD3DX12_RASTERIZER_DESC {
    fn default() -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            FrontCounterClockwise: FALSE,
            DepthBias: D3D12_DEFAULT_DEPTH_BIAS,
            DepthBiasClamp: D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
            SlopeScaledDepthBias: D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
            DepthClipEnable: TRUE,
            MultisampleEnable: FALSE,
            AntialiasedLineEnable: FALSE,
            ForcedSampleCount: 0,
            ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
        }
    }
    fn new(
        fill_mode: D3D12_FILL_MODE,
        cull_mode: D3D12_CULL_MODE,
        front_counter_clockwise: BOOL,
        depth_bias: i32,
        depth_bias_clamp: f32,
        slope_scaled_depth_bias: f32,
        depth_clip_enable: BOOL,
        multisample_enable: BOOL,
        antialiased_line_enable: BOOL,
        forced_sample_count: u32,
        conservative_raster: D3D12_CONSERVATIVE_RASTERIZATION_MODE,
    ) -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: fill_mode,
            CullMode: cull_mode,
            FrontCounterClockwise: front_counter_clockwise,
            DepthBias: depth_bias,
            DepthBiasClamp: depth_bias_clamp,
            SlopeScaledDepthBias: slope_scaled_depth_bias,
            DepthClipEnable: depth_clip_enable,
            MultisampleEnable: multisample_enable,
            AntialiasedLineEnable: antialiased_line_enable,
            ForcedSampleCount: forced_sample_count,
            ConservativeRaster: conservative_raster,
        }
    }
}

// //------------------------------------------------------------------------------------------------
// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 608)
// struct CD3DX12_RASTERIZER_DESC1 : public D3D12_RASTERIZER_DESC1
// {
//     CD3DX12_RASTERIZER_DESC1() = default;
//     explicit CD3DX12_RASTERIZER_DESC1(const D3D12_RASTERIZER_DESC1& o) noexcept :
//         D3D12_RASTERIZER_DESC1(o)

//     {
//     }
//     explicit CD3DX12_RASTERIZER_DESC1(const D3D12_RASTERIZER_DESC& o) noexcept
//     {
//         FillMode = o.FillMode;
//         CullMode = o.CullMode;
//         FrontCounterClockwise = o.FrontCounterClockwise;
//         DepthBias = static_cast<FLOAT>(o.DepthBias);
//         DepthBiasClamp = o.DepthBiasClamp;
//         SlopeScaledDepthBias = o.SlopeScaledDepthBias;
//         DepthClipEnable = o.DepthClipEnable;
//         MultisampleEnable = o.MultisampleEnable;
//         AntialiasedLineEnable = o.AntialiasedLineEnable;
//         ForcedSampleCount = o.ForcedSampleCount;
//         ConservativeRaster = o.ConservativeRaster;
//     }
//     explicit CD3DX12_RASTERIZER_DESC1(CD3DX12_DEFAULT) noexcept
//     {
//         FillMode = D3D12_FILL_MODE_SOLID;
//         CullMode = D3D12_CULL_MODE_BACK;
//         FrontCounterClockwise = FALSE;
//         DepthBias = D3D12_DEFAULT_DEPTH_BIAS;
//         DepthBiasClamp = D3D12_DEFAULT_DEPTH_BIAS_CLAMP;
//         SlopeScaledDepthBias = D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS;
//         DepthClipEnable = TRUE;
//         MultisampleEnable = FALSE;
//         AntialiasedLineEnable = FALSE;
//         ForcedSampleCount = 0;
//         ConservativeRaster = D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF;
//     }
//     explicit CD3DX12_RASTERIZER_DESC1(
//         D3D12_FILL_MODE fillMode,
//         D3D12_CULL_MODE cullMode,
//         BOOL frontCounterClockwise,
//         FLOAT depthBias,
//         FLOAT depthBiasClamp,
//         FLOAT slopeScaledDepthBias,
//         BOOL depthClipEnable,
//         BOOL multisampleEnable,
//         BOOL antialiasedLineEnable,
//         UINT forcedSampleCount,
//         D3D12_CONSERVATIVE_RASTERIZATION_MODE conservativeRaster) noexcept
//     {
//         FillMode = fillMode;
//         CullMode = cullMode;
//         FrontCounterClockwise = frontCounterClockwise;
//         DepthBias = depthBias;
//         DepthBiasClamp = depthBiasClamp;
//         SlopeScaledDepthBias = slopeScaledDepthBias;
//         DepthClipEnable = depthClipEnable;
//         MultisampleEnable = multisampleEnable;
//         AntialiasedLineEnable = antialiasedLineEnable;
//         ForcedSampleCount = forcedSampleCount;
//         ConservativeRaster = conservativeRaster;
//     }

//     operator D3D12_RASTERIZER_DESC() const noexcept
//     {
//         D3D12_RASTERIZER_DESC o;

//         o.FillMode = FillMode;
//         o.CullMode = CullMode;
//         o.FrontCounterClockwise = FrontCounterClockwise;
//         o.DepthBias = static_cast<INT>(DepthBias);
//         o.DepthBiasClamp = DepthBiasClamp;
//         o.SlopeScaledDepthBias = SlopeScaledDepthBias;
//         o.DepthClipEnable = DepthClipEnable;
//         o.MultisampleEnable = MultisampleEnable;
//         o.AntialiasedLineEnable = AntialiasedLineEnable;
//         o.ForcedSampleCount = ForcedSampleCount;
//         o.ConservativeRaster = ConservativeRaster;

//         return o;
//     }
// };
// #endif // D3D12_SDK_VERSION >= 608

// //------------------------------------------------------------------------------------------------
// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 610)
// struct CD3DX12_RASTERIZER_DESC2 : public D3D12_RASTERIZER_DESC2
// {
//     CD3DX12_RASTERIZER_DESC2() = default;
//     explicit CD3DX12_RASTERIZER_DESC2(const D3D12_RASTERIZER_DESC2& o) noexcept :
//         D3D12_RASTERIZER_DESC2(o)

//     {
//     }
//     explicit CD3DX12_RASTERIZER_DESC2(const D3D12_RASTERIZER_DESC1& o) noexcept
//     {
//         FillMode = o.FillMode;
//         CullMode = o.CullMode;
//         FrontCounterClockwise = o.FrontCounterClockwise;
//         DepthBias = o.DepthBias;
//         DepthBiasClamp = o.DepthBiasClamp;
//         SlopeScaledDepthBias = o.SlopeScaledDepthBias;
//         DepthClipEnable = o.DepthClipEnable;
//         LineRasterizationMode = D3D12_LINE_RASTERIZATION_MODE_ALIASED;
//         if (o.MultisampleEnable)
//         {
//             LineRasterizationMode = D3D12_LINE_RASTERIZATION_MODE_QUADRILATERAL_WIDE;
//         }
//         else if (o.AntialiasedLineEnable)
//         {
//             LineRasterizationMode = D3D12_LINE_RASTERIZATION_MODE_ALPHA_ANTIALIASED;
//         }
//         ForcedSampleCount = o.ForcedSampleCount;
//         ConservativeRaster = o.ConservativeRaster;
//     }
//     explicit CD3DX12_RASTERIZER_DESC2(const D3D12_RASTERIZER_DESC& o) noexcept
//         : CD3DX12_RASTERIZER_DESC2(CD3DX12_RASTERIZER_DESC1(o))
//     {
//     }
//     explicit CD3DX12_RASTERIZER_DESC2(CD3DX12_DEFAULT) noexcept
//     {
//         FillMode = D3D12_FILL_MODE_SOLID;
//         CullMode = D3D12_CULL_MODE_BACK;
//         FrontCounterClockwise = FALSE;
//         DepthBias = D3D12_DEFAULT_DEPTH_BIAS;
//         DepthBiasClamp = D3D12_DEFAULT_DEPTH_BIAS_CLAMP;
//         SlopeScaledDepthBias = D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS;
//         DepthClipEnable = TRUE;
//         LineRasterizationMode = D3D12_LINE_RASTERIZATION_MODE_ALIASED;
//         ForcedSampleCount = 0;
//         ConservativeRaster = D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF;
//     }
//     explicit CD3DX12_RASTERIZER_DESC2(
//         D3D12_FILL_MODE fillMode,
//         D3D12_CULL_MODE cullMode,
//         BOOL frontCounterClockwise,
//         FLOAT depthBias,
//         FLOAT depthBiasClamp,
//         FLOAT slopeScaledDepthBias,
//         BOOL depthClipEnable,
//         D3D12_LINE_RASTERIZATION_MODE lineRasterizationMode,
//         UINT forcedSampleCount,
//         D3D12_CONSERVATIVE_RASTERIZATION_MODE conservativeRaster) noexcept
//     {
//         FillMode = fillMode;
//         CullMode = cullMode;
//         FrontCounterClockwise = frontCounterClockwise;
//         DepthBias = depthBias;
//         DepthBiasClamp = depthBiasClamp;
//         SlopeScaledDepthBias = slopeScaledDepthBias;
//         DepthClipEnable = depthClipEnable;
//         LineRasterizationMode = lineRasterizationMode;
//         ForcedSampleCount = forcedSampleCount;
//         ConservativeRaster = conservativeRaster;
//     }

//     operator D3D12_RASTERIZER_DESC1() const noexcept
//     {
//         D3D12_RASTERIZER_DESC1 o;

//         o.FillMode = FillMode;
//         o.CullMode = CullMode;
//         o.FrontCounterClockwise = FrontCounterClockwise;
//         o.DepthBias = DepthBias;
//         o.DepthBiasClamp = DepthBiasClamp;
//         o.SlopeScaledDepthBias = SlopeScaledDepthBias;
//         o.DepthClipEnable = DepthClipEnable;
//         o.MultisampleEnable = FALSE;
//         o.AntialiasedLineEnable = FALSE;
//         if (LineRasterizationMode == D3D12_LINE_RASTERIZATION_MODE_ALPHA_ANTIALIASED)
//         {
//             o.AntialiasedLineEnable = TRUE;
//         }
//         else if (LineRasterizationMode != D3D12_LINE_RASTERIZATION_MODE_ALIASED)
//         {
//             o.MultisampleEnable = TRUE;
//         }
//         o.ForcedSampleCount = ForcedSampleCount;
//         o.ConservativeRaster = ConservativeRaster;

//         return o;
//     }
//     operator D3D12_RASTERIZER_DESC() const noexcept
//     {
//         return (D3D12_RASTERIZER_DESC)CD3DX12_RASTERIZER_DESC1((D3D12_RASTERIZER_DESC1)*this);
//     }
// };
// #endif // D3D12_SDK_VERSION >= 610

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_RESOURCE_ALLOCATION_INFO : public D3D12_RESOURCE_ALLOCATION_INFO
// {
//     CD3DX12_RESOURCE_ALLOCATION_INFO() = default;
//     explicit CD3DX12_RESOURCE_ALLOCATION_INFO( const D3D12_RESOURCE_ALLOCATION_INFO& o ) noexcept :
//         D3D12_RESOURCE_ALLOCATION_INFO( o )
//     {}
//     CD3DX12_RESOURCE_ALLOCATION_INFO(
//         UINT64 size,
//         UINT64 alignment ) noexcept
//     {
//         SizeInBytes = size;
//         Alignment = alignment;
//     }
// };

// //------------------------------------------------------------------------------------------------
pub trait CD3DX12_HEAP_PROPERTIES {
    fn new_custom(
        cpu_page_property: D3D12_CPU_PAGE_PROPERTY,
        memory_pool_preference: D3D12_MEMORY_POOL,
        creation_node_mask: Option<u32>,
        node_mask: Option<u32>,
    ) -> D3D12_HEAP_PROPERTIES {
        let creation_node_mask = creation_node_mask.unwrap_or(1);
        let node_mask = node_mask.unwrap_or(1);
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_CUSTOM,
            CPUPageProperty: cpu_page_property,
            MemoryPoolPreference: memory_pool_preference,
            CreationNodeMask: creation_node_mask,
            VisibleNodeMask: node_mask,
        }
    }

    fn new_with_type(
        r#type: D3D12_HEAP_TYPE,
        creation_node_mask: Option<u32>,
        node_mask: Option<u32>,
    ) -> D3D12_HEAP_PROPERTIES {
        let creation_node_mask = creation_node_mask.unwrap_or(1);
        let node_mask = node_mask.unwrap_or(1);
        D3D12_HEAP_PROPERTIES {
            Type: r#type,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: creation_node_mask,
            VisibleNodeMask: node_mask,
        }
    }
    fn is_cpu_accessible(&self) -> bool;
}
impl CD3DX12_HEAP_PROPERTIES for D3D12_HEAP_PROPERTIES {
    fn is_cpu_accessible(&self) -> bool {
        #[allow(unused_mut)]
        let mut result = self.Type == D3D12_HEAP_TYPE_UPLOAD
            || self.Type == D3D12_HEAP_TYPE_READBACK
            || (self.Type == D3D12_HEAP_TYPE_CUSTOM
                && (self.CPUPageProperty == D3D12_CPU_PAGE_PROPERTY_WRITE_COMBINE
                    || self.CPUPageProperty == D3D12_CPU_PAGE_PROPERTY_WRITE_BACK));
        #[cfg(feature = "d3d12_sdk_version_gte_609")]
        {
            result = result || self.Type == D3D12_HEAP_TYPE_GPU_UPLOAD;
        }
        result
    }
}

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_HEAP_DESC : public D3D12_HEAP_DESC
// {
//     CD3DX12_HEAP_DESC() = default;
//     explicit CD3DX12_HEAP_DESC(const D3D12_HEAP_DESC &o) noexcept :
//         D3D12_HEAP_DESC(o)
//     {}
//     CD3DX12_HEAP_DESC(
//         UINT64 size,
//         D3D12_HEAP_PROPERTIES properties,
//         UINT64 alignment = 0,
//         D3D12_HEAP_FLAGS flags = D3D12_HEAP_FLAG_NONE ) noexcept
//     {
//         SizeInBytes = size;
//         Properties = properties;
//         Alignment = alignment;
//         Flags = flags;
//     }
//     CD3DX12_HEAP_DESC(
//         UINT64 size,
//         D3D12_HEAP_TYPE type,
//         UINT64 alignment = 0,
//         D3D12_HEAP_FLAGS flags = D3D12_HEAP_FLAG_NONE ) noexcept
//     {
//         SizeInBytes = size;
//         Properties = CD3DX12_HEAP_PROPERTIES( type );
//         Alignment = alignment;
//         Flags = flags;
//     }
//     CD3DX12_HEAP_DESC(
//         UINT64 size,
//         D3D12_CPU_PAGE_PROPERTY cpuPageProperty,
//         D3D12_MEMORY_POOL memoryPoolPreference,
//         UINT64 alignment = 0,
//         D3D12_HEAP_FLAGS flags = D3D12_HEAP_FLAG_NONE ) noexcept
//     {
//         SizeInBytes = size;
//         Properties = CD3DX12_HEAP_PROPERTIES( cpuPageProperty, memoryPoolPreference );
//         Alignment = alignment;
//         Flags = flags;
//     }
//     CD3DX12_HEAP_DESC(
//         const D3D12_RESOURCE_ALLOCATION_INFO& resAllocInfo,
//         D3D12_HEAP_PROPERTIES properties,
//         D3D12_HEAP_FLAGS flags = D3D12_HEAP_FLAG_NONE ) noexcept
//     {
//         SizeInBytes = resAllocInfo.SizeInBytes;
//         Properties = properties;
//         Alignment = resAllocInfo.Alignment;
//         Flags = flags;
//     }
//     CD3DX12_HEAP_DESC(
//         const D3D12_RESOURCE_ALLOCATION_INFO& resAllocInfo,
//         D3D12_HEAP_TYPE type,
//         D3D12_HEAP_FLAGS flags = D3D12_HEAP_FLAG_NONE ) noexcept
//     {
//         SizeInBytes = resAllocInfo.SizeInBytes;
//         Properties = CD3DX12_HEAP_PROPERTIES( type );
//         Alignment = resAllocInfo.Alignment;
//         Flags = flags;
//     }
//     CD3DX12_HEAP_DESC(
//         const D3D12_RESOURCE_ALLOCATION_INFO& resAllocInfo,
//         D3D12_CPU_PAGE_PROPERTY cpuPageProperty,
//         D3D12_MEMORY_POOL memoryPoolPreference,
//         D3D12_HEAP_FLAGS flags = D3D12_HEAP_FLAG_NONE ) noexcept
//     {
//         SizeInBytes = resAllocInfo.SizeInBytes;
//         Properties = CD3DX12_HEAP_PROPERTIES( cpuPageProperty, memoryPoolPreference );
//         Alignment = resAllocInfo.Alignment;
//         Flags = flags;
//     }
//     bool IsCPUAccessible() const noexcept
//     { return static_cast< const CD3DX12_HEAP_PROPERTIES* >( &Properties )->IsCPUAccessible(); }
// };
// inline bool operator==( const D3D12_HEAP_DESC& l, const D3D12_HEAP_DESC& r ) noexcept
// {
//     return l.SizeInBytes == r.SizeInBytes &&
//         l.Properties == r.Properties &&
//         l.Alignment == r.Alignment &&
//         l.Flags == r.Flags;
// }
// inline bool operator!=( const D3D12_HEAP_DESC& l, const D3D12_HEAP_DESC& r ) noexcept
// { return !( l == r ); }

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_CLEAR_VALUE : public D3D12_CLEAR_VALUE
// {
//     CD3DX12_CLEAR_VALUE() = default;
//     explicit CD3DX12_CLEAR_VALUE(const D3D12_CLEAR_VALUE &o) noexcept :
//         D3D12_CLEAR_VALUE(o)
//     {}
//     CD3DX12_CLEAR_VALUE(
//         DXGI_FORMAT format,
//         const FLOAT color[4] ) noexcept
//     {
//         Format = format;
//         memcpy( Color, color, sizeof( Color ) );
//     }
//     CD3DX12_CLEAR_VALUE(
//         DXGI_FORMAT format,
//         FLOAT depth,
//         UINT8 stencil ) noexcept
//     {
//         Format = format;
//         memset( &Color, 0, sizeof( Color ) );
//         /* Use memcpy to preserve NAN values */
//         memcpy( &DepthStencil.Depth, &depth, sizeof( depth ) );
//         DepthStencil.Stencil = stencil;
//     }
// };

// //------------------------------------------------------------------------------------------------
// inline bool operator==( const D3D12_CLEAR_VALUE &a, const D3D12_CLEAR_VALUE &b) noexcept
// {
//     if (a.Format != b.Format) return false;
//     if (a.Format == DXGI_FORMAT_D24_UNORM_S8_UINT
//      || a.Format == DXGI_FORMAT_D16_UNORM
//      || a.Format == DXGI_FORMAT_D32_FLOAT
//      || a.Format == DXGI_FORMAT_D32_FLOAT_S8X24_UINT)
//     {
//         return (a.DepthStencil.Depth == b.DepthStencil.Depth) &&
//           (a.DepthStencil.Stencil == b.DepthStencil.Stencil);
//     } else {
//         return (a.Color[0] == b.Color[0]) &&
//                (a.Color[1] == b.Color[1]) &&
//                (a.Color[2] == b.Color[2]) &&
//                (a.Color[3] == b.Color[3]);
//     }
// }

//------------------------------------------------------------------------------------------------

#[repr(transparent)]
pub struct CD3DX12_RANGE(pub D3D12_RANGE);
impl ::core::marker::Copy for CD3DX12_RANGE {}
impl ::core::clone::Clone for CD3DX12_RANGE {
    fn clone(&self) -> Self {
        *self
    }
}

impl CD3DX12_RANGE {
    pub fn new(begin: usize, end: usize) -> Self {
        Self(D3D12_RANGE {
            Begin: begin,
            End: end,
        })
    }
}

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_RANGE_UINT64 : public D3D12_RANGE_UINT64
// {
//     CD3DX12_RANGE_UINT64() = default;
//     explicit CD3DX12_RANGE_UINT64(const D3D12_RANGE_UINT64 &o) noexcept :
//         D3D12_RANGE_UINT64(o)
//     {}
//     CD3DX12_RANGE_UINT64(
//         UINT64 begin,
//         UINT64 end ) noexcept
//     {
//         Begin = begin;
//         End = end;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_SUBRESOURCE_RANGE_UINT64 : public D3D12_SUBRESOURCE_RANGE_UINT64
// {
//     CD3DX12_SUBRESOURCE_RANGE_UINT64() = default;
//     explicit CD3DX12_SUBRESOURCE_RANGE_UINT64(const D3D12_SUBRESOURCE_RANGE_UINT64 &o) noexcept :
//         D3D12_SUBRESOURCE_RANGE_UINT64(o)
//     {}
//     CD3DX12_SUBRESOURCE_RANGE_UINT64(
//         UINT subresource,
//         const D3D12_RANGE_UINT64& range ) noexcept
//     {
//         Subresource = subresource;
//         Range = range;
//     }
//     CD3DX12_SUBRESOURCE_RANGE_UINT64(
//         UINT subresource,
//         UINT64 begin,
//         UINT64 end ) noexcept
//     {
//         Subresource = subresource;
//         Range.Begin = begin;
//         Range.End = end;
//     }
// };

// //------------------------------------------------------------------------------------------------
#[allow(non_camel_case_types)]
pub trait CD3DX12_SHADER_BYTECODE {
    #[allow(non_snake_case)]
    fn from_ID3DBlob(shader_blob: &ID3DBlob) -> D3D12_SHADER_BYTECODE {
        D3D12_SHADER_BYTECODE {
            pShaderBytecode: unsafe { shader_blob.GetBufferPointer() },
            BytecodeLength: unsafe { shader_blob.GetBufferSize() },
        }
    }
    #[allow(clippy::new_ret_no_self)]
    fn new(shader_bytecode: *const c_void, bytecode_length: usize) -> D3D12_SHADER_BYTECODE {
        D3D12_SHADER_BYTECODE {
            pShaderBytecode: shader_bytecode,
            BytecodeLength: bytecode_length,
        }
    }
}

impl CD3DX12_SHADER_BYTECODE for D3D12_SHADER_BYTECODE {}
// {
//     CD3DX12_SHADER_BYTECODE() = default;
//     explicit CD3DX12_SHADER_BYTECODE(const D3D12_SHADER_BYTECODE &o) noexcept :
//         D3D12_SHADER_BYTECODE(o)
//     {}
//     CD3DX12_SHADER_BYTECODE(
//         _In_ ID3DBlob* pShaderBlob ) noexcept
//     {
//         pShaderBytecode = pShaderBlob->GetBufferPointer();
//         BytecodeLength = pShaderBlob->GetBufferSize();
//     }
//     CD3DX12_SHADER_BYTECODE(
//         const void* _pShaderBytecode,
//         SIZE_T bytecodeLength ) noexcept
//     {
//         pShaderBytecode = _pShaderBytecode;
//         BytecodeLength = bytecodeLength;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_TILED_RESOURCE_COORDINATE : public D3D12_TILED_RESOURCE_COORDINATE
// {
//     CD3DX12_TILED_RESOURCE_COORDINATE() = default;
//     explicit CD3DX12_TILED_RESOURCE_COORDINATE(const D3D12_TILED_RESOURCE_COORDINATE &o) noexcept :
//         D3D12_TILED_RESOURCE_COORDINATE(o)
//     {}
//     CD3DX12_TILED_RESOURCE_COORDINATE(
//         UINT x,
//         UINT y,
//         UINT z,
//         UINT subresource ) noexcept
//     {
//         X = x;
//         Y = y;
//         Z = z;
//         Subresource = subresource;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_TILE_REGION_SIZE : public D3D12_TILE_REGION_SIZE
// {
//     CD3DX12_TILE_REGION_SIZE() = default;
//     explicit CD3DX12_TILE_REGION_SIZE(const D3D12_TILE_REGION_SIZE &o) noexcept :
//         D3D12_TILE_REGION_SIZE(o)
//     {}
//     CD3DX12_TILE_REGION_SIZE(
//         UINT numTiles,
//         BOOL useBox,
//         UINT width,
//         UINT16 height,
//         UINT16 depth ) noexcept
//     {
//         NumTiles = numTiles;
//         UseBox = useBox;
//         Width = width;
//         Height = height;
//         Depth = depth;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_SUBRESOURCE_TILING : public D3D12_SUBRESOURCE_TILING
// {
//     CD3DX12_SUBRESOURCE_TILING() = default;
//     explicit CD3DX12_SUBRESOURCE_TILING(const D3D12_SUBRESOURCE_TILING &o) noexcept :
//         D3D12_SUBRESOURCE_TILING(o)
//     {}
//     CD3DX12_SUBRESOURCE_TILING(
//         UINT widthInTiles,
//         UINT16 heightInTiles,
//         UINT16 depthInTiles,
//         UINT startTileIndexInOverallResource ) noexcept
//     {
//         WidthInTiles = widthInTiles;
//         HeightInTiles = heightInTiles;
//         DepthInTiles = depthInTiles;
//         StartTileIndexInOverallResource = startTileIndexInOverallResource;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_TILE_SHAPE : public D3D12_TILE_SHAPE
// {
//     CD3DX12_TILE_SHAPE() = default;
//     explicit CD3DX12_TILE_SHAPE(const D3D12_TILE_SHAPE &o) noexcept :
//         D3D12_TILE_SHAPE(o)
//     {}
//     CD3DX12_TILE_SHAPE(
//         UINT widthInTexels,
//         UINT heightInTexels,
//         UINT depthInTexels ) noexcept
//     {
//         WidthInTexels = widthInTexels;
//         HeightInTexels = heightInTexels;
//         DepthInTexels = depthInTexels;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_PACKED_MIP_INFO : public D3D12_PACKED_MIP_INFO
// {
//     CD3DX12_PACKED_MIP_INFO() = default;
//     explicit CD3DX12_PACKED_MIP_INFO(const D3D12_PACKED_MIP_INFO &o) noexcept :
//         D3D12_PACKED_MIP_INFO(o)
//     {}
//     CD3DX12_PACKED_MIP_INFO(
//         UINT8 numStandardMips,
//         UINT8 numPackedMips,
//         UINT numTilesForPackedMips,
//         UINT startTileIndexInOverallResource ) noexcept
//     {
//         NumStandardMips = numStandardMips;
//         NumPackedMips = numPackedMips;
//         NumTilesForPackedMips = numTilesForPackedMips;
//         StartTileIndexInOverallResource = startTileIndexInOverallResource;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_SUBRESOURCE_FOOTPRINT : public D3D12_SUBRESOURCE_FOOTPRINT
// {
//     CD3DX12_SUBRESOURCE_FOOTPRINT() = default;
//     explicit CD3DX12_SUBRESOURCE_FOOTPRINT(const D3D12_SUBRESOURCE_FOOTPRINT &o) noexcept :
//         D3D12_SUBRESOURCE_FOOTPRINT(o)
//     {}
//     CD3DX12_SUBRESOURCE_FOOTPRINT(
//         DXGI_FORMAT format,
//         UINT width,
//         UINT height,
//         UINT depth,
//         UINT rowPitch ) noexcept
//     {
//         Format = format;
//         Width = width;
//         Height = height;
//         Depth = depth;
//         RowPitch = rowPitch;
//     }
//     explicit CD3DX12_SUBRESOURCE_FOOTPRINT(
//         const D3D12_RESOURCE_DESC& resDesc,
//         UINT rowPitch ) noexcept
//     {
//         Format = resDesc.Format;
//         Width = UINT( resDesc.Width );
//         Height = resDesc.Height;
//         Depth = (resDesc.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D ? resDesc.DepthOrArraySize : 1u);
//         RowPitch = rowPitch;
//     }
// };

//------------------------------------------------------------------------------------------------
pub struct CD3DX12_TEXTURE_COPY_LOCATION(pub D3D12_TEXTURE_COPY_LOCATION);
impl CD3DX12_TEXTURE_COPY_LOCATION {
    // CD3DX12_TEXTURE_COPY_LOCATION() = default;
    // explicit CD3DX12_TEXTURE_COPY_LOCATION(const D3D12_TEXTURE_COPY_LOCATION &o) noexcept :
    //     D3D12_TEXTURE_COPY_LOCATION(o)
    // {}
    // CD3DX12_TEXTURE_COPY_LOCATION(_In_ ID3D12Resource* pRes) noexcept
    // {
    //     pResource = pRes;
    //     Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    //     PlacedFootprint = {};
    // }
    // CD3DX12_TEXTURE_COPY_LOCATION(_In_ ID3D12Resource* pRes, D3D12_PLACED_SUBRESOURCE_FOOTPRINT const& Footprint) noexcept
    pub fn new_with_placed_footprint(
        pRes: &ID3D12Resource,
        Footprint: &D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    ) -> Self {
        Self(D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(pRes) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: *Footprint,
            },
        })
    }

    // pub fn CD3DX12_TEXTURE_COPY_LOCATION(pRes: &ID3D12Resource, Sub: u32) -> Self {
    pub fn new_with_subresource_index(pRes: &ID3D12Resource, Sub: u32) -> Self {
        Self(D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(pRes) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: Sub,
            },
        })
    }
}

// //------------------------------------------------------------------------------------------------
// constexpr UINT D3D12CalcSubresource( UINT MipSlice, UINT ArraySlice, UINT PlaneSlice, UINT MipLevels, UINT ArraySize ) noexcept
// {
//     return MipSlice + ArraySlice * MipLevels + PlaneSlice * MipLevels * ArraySize;
// }

// //------------------------------------------------------------------------------------------------
// inline UINT8 D3D12GetFormatPlaneCount(
//     _In_ ID3D12Device* pDevice,
//     DXGI_FORMAT Format
//     ) noexcept
// {
//     D3D12_FEATURE_DATA_FORMAT_INFO formatInfo = { Format, 0 };
//     if (FAILED(pDevice->CheckFeatureSupport(D3D12_FEATURE_FORMAT_INFO, &formatInfo, sizeof(formatInfo))))
//     {
//         return 0;
//     }
//     return formatInfo.PlaneCount;
// }

//------------------------------------------------------------------------------------------------
#[allow(non_camel_case_types)]
pub trait CD3DX12_RESOURCE_DESC {
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::new_ret_no_self)]
    #[inline]
    fn new(
        dimension: D3D12_RESOURCE_DIMENSION,
        alignment: u64,
        width: u64,
        height: u32,
        depth_or_array_size: u16,
        mip_levels: u16,
        format: DXGI_FORMAT,
        sample_count: u32,
        sample_quality: u32,
        layout: D3D12_TEXTURE_LAYOUT,
        flags: D3D12_RESOURCE_FLAGS,
    ) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: dimension,
            Alignment: alignment,
            Width: width,
            Height: height,
            DepthOrArraySize: depth_or_array_size,
            MipLevels: mip_levels,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: sample_count,
                Quality: sample_quality,
            },
            Layout: layout,
            Flags: flags,
        }
    }

    #[inline]
    fn buffer(width: u64) -> D3D12_RESOURCE_DESC {
        let flags = D3D12_RESOURCE_FLAG_NONE;
        let alignment = 0;
        Self::new(
            D3D12_RESOURCE_DIMENSION_BUFFER,
            alignment,
            width,
            1,
            1,
            1,
            DXGI_FORMAT_UNKNOWN,
            1,
            0,
            D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            flags,
        )
    }
    #[allow(clippy::too_many_arguments)]
    #[inline]
    fn tex_2d(
        format: DXGI_FORMAT,
        width: u64,
        height: u32,
        array_size: Option<u16>,
        mip_levels: Option<u16>,
        sample_count: Option<u32>,
        sample_quality: Option<u32>,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        layout: Option<D3D12_TEXTURE_LAYOUT>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC {
        let array_size = array_size.unwrap_or(1);
        let mip_levels = mip_levels.unwrap_or(0);
        let sample_count = sample_count.unwrap_or(1);
        let sample_quality = sample_quality.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let layout = layout.unwrap_or(D3D12_TEXTURE_LAYOUT_UNKNOWN);
        let alignment = alignment.unwrap_or(0);
        Self::new(
            D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            alignment,
            width,
            height,
            array_size,
            mip_levels,
            format,
            sample_count,
            sample_quality,
            layout,
            flags,
        )
    }
    // CD3DX12_RESOURCE_DESC() = default;
    // explicit CD3DX12_RESOURCE_DESC( const D3D12_RESOURCE_DESC& o ) noexcept :
    //     D3D12_RESOURCE_DESC( o )
    // {}
    // CD3DX12_RESOURCE_DESC(
    //     D3D12_RESOURCE_DIMENSION dimension,
    //     UINT64 alignment,
    //     UINT64 width,
    //     UINT height,
    //     UINT16 depthOrArraySize,
    //     UINT16 mipLevels,
    //     DXGI_FORMAT format,
    //     UINT sampleCount,
    //     UINT sampleQuality,
    //     D3D12_TEXTURE_LAYOUT layout,
    //     D3D12_RESOURCE_FLAGS flags ) noexcept
    // {
    //     Dimension = dimension;
    //     Alignment = alignment;
    //     Width = width;
    //     Height = height;
    //     DepthOrArraySize = depthOrArraySize;
    //     MipLevels = mipLevels;
    //     Format = format;
    //     SampleDesc.Count = sampleCount;
    //     SampleDesc.Quality = sampleQuality;
    //     Layout = layout;
    //     Flags = flags;
    // }
    // static inline CD3DX12_RESOURCE_DESC Buffer(
    //     const D3D12_RESOURCE_ALLOCATION_INFO& resAllocInfo,
    //     D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE ) noexcept
    // {
    //     return CD3DX12_RESOURCE_DESC( D3D12_RESOURCE_DIMENSION_BUFFER, resAllocInfo.Alignment, resAllocInfo.SizeInBytes,
    //         1, 1, 1, DXGI_FORMAT_UNKNOWN, 1, 0, D3D12_TEXTURE_LAYOUT_ROW_MAJOR, flags );
    // }
    // static inline CD3DX12_RESOURCE_DESC Buffer(
    //     UINT64 width,
    //     D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
    //     UINT64 alignment = 0 ) noexcept
    // {
    //     return CD3DX12_RESOURCE_DESC( D3D12_RESOURCE_DIMENSION_BUFFER, alignment, width, 1, 1, 1,
    //         DXGI_FORMAT_UNKNOWN, 1, 0, D3D12_TEXTURE_LAYOUT_ROW_MAJOR, flags );
    // }
    // static inline CD3DX12_RESOURCE_DESC Tex1D(
    //     DXGI_FORMAT format,
    //     UINT64 width,
    //     UINT16 arraySize = 1,
    //     UINT16 mipLevels = 0,
    //     D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
    //     D3D12_TEXTURE_LAYOUT layout = D3D12_TEXTURE_LAYOUT_UNKNOWN,
    //     UINT64 alignment = 0 ) noexcept
    // {
    //     return CD3DX12_RESOURCE_DESC( D3D12_RESOURCE_DIMENSION_TEXTURE1D, alignment, width, 1, arraySize,
    //         mipLevels, format, 1, 0, layout, flags );
    // }
    // static inline CD3DX12_RESOURCE_DESC Tex2D(
    //     DXGI_FORMAT format,
    //     UINT64 width,
    //     UINT height,
    //     UINT16 arraySize = 1,
    //     UINT16 mipLevels = 0,
    //     UINT sampleCount = 1,
    //     UINT sampleQuality = 0,
    //     D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
    //     D3D12_TEXTURE_LAYOUT layout = D3D12_TEXTURE_LAYOUT_UNKNOWN,
    //     UINT64 alignment = 0 ) noexcept
    // {
    //     return CD3DX12_RESOURCE_DESC( D3D12_RESOURCE_DIMENSION_TEXTURE2D, alignment, width, height, arraySize,
    //         mipLevels, format, sampleCount, sampleQuality, layout, flags );
    // }
    // static inline CD3DX12_RESOURCE_DESC Tex3D(
    //     DXGI_FORMAT format,
    //     UINT64 width,
    //     UINT height,
    //     UINT16 depth,
    //     UINT16 mipLevels = 0,
    //     D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
    //     D3D12_TEXTURE_LAYOUT layout = D3D12_TEXTURE_LAYOUT_UNKNOWN,
    //     UINT64 alignment = 0 ) noexcept
    // {
    //     return CD3DX12_RESOURCE_DESC( D3D12_RESOURCE_DIMENSION_TEXTURE3D, alignment, width, height, depth,
    //         mipLevels, format, 1, 0, layout, flags );
    // }
    // inline UINT16 Depth() const noexcept
    // { return (Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D ? DepthOrArraySize : 1u); }
    // inline UINT16 ArraySize() const noexcept
    // { return (Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE3D ? DepthOrArraySize : 1u); }
    // inline UINT8 PlaneCount(_In_ ID3D12Device* pDevice) const noexcept
    // { return D3D12GetFormatPlaneCount(pDevice, Format); }
    // inline UINT Subresources(_In_ ID3D12Device* pDevice) const noexcept
    // { return static_cast<UINT>(MipLevels) * ArraySize() * PlaneCount(pDevice); }
    // inline UINT CalcSubresource(UINT MipSlice, UINT ArraySlice, UINT PlaneSlice) noexcept
    // { return D3D12CalcSubresource(MipSlice, ArraySlice, PlaneSlice, MipLevels, ArraySize()); }
}
impl CD3DX12_RESOURCE_DESC for D3D12_RESOURCE_DESC {}

// inline bool operator==( const D3D12_RESOURCE_DESC& l, const D3D12_RESOURCE_DESC& r ) noexcept
// {
//     return l.Dimension == r.Dimension &&
//         l.Alignment == r.Alignment &&
//         l.Width == r.Width &&
//         l.Height == r.Height &&
//         l.DepthOrArraySize == r.DepthOrArraySize &&
//         l.MipLevels == r.MipLevels &&
//         l.Format == r.Format &&
//         l.SampleDesc.Count == r.SampleDesc.Count &&
//         l.SampleDesc.Quality == r.SampleDesc.Quality &&
//         l.Layout == r.Layout &&
//         l.Flags == r.Flags;
// }
// inline bool operator!=( const D3D12_RESOURCE_DESC& l, const D3D12_RESOURCE_DESC& r ) noexcept
// { return !( l == r ); }

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_RESOURCE_DESC1 : public D3D12_RESOURCE_DESC1
// {
//     CD3DX12_RESOURCE_DESC1() = default;
//     explicit CD3DX12_RESOURCE_DESC1( const D3D12_RESOURCE_DESC1& o ) noexcept :
//         D3D12_RESOURCE_DESC1( o )
//     {}
//     explicit CD3DX12_RESOURCE_DESC1( const D3D12_RESOURCE_DESC& o ) noexcept
//     {
//         Dimension = o.Dimension;
//         Alignment = o.Alignment;
//         Width = o.Width;
//         Height = o.Height;
//         DepthOrArraySize = o.DepthOrArraySize;
//         MipLevels = o.MipLevels;
//         Format = o.Format;
//         SampleDesc = o.SampleDesc;
//         Layout = o.Layout;
//         Flags = o.Flags;
//         SamplerFeedbackMipRegion = {};
//     }
//     CD3DX12_RESOURCE_DESC1(
//         D3D12_RESOURCE_DIMENSION dimension,
//         UINT64 alignment,
//         UINT64 width,
//         UINT height,
//         UINT16 depthOrArraySize,
//         UINT16 mipLevels,
//         DXGI_FORMAT format,
//         UINT sampleCount,
//         UINT sampleQuality,
//         D3D12_TEXTURE_LAYOUT layout,
//         D3D12_RESOURCE_FLAGS flags,
//         UINT samplerFeedbackMipRegionWidth = 0,
//         UINT samplerFeedbackMipRegionHeight = 0,
//         UINT samplerFeedbackMipRegionDepth = 0) noexcept
//     {
//         Dimension = dimension;
//         Alignment = alignment;
//         Width = width;
//         Height = height;
//         DepthOrArraySize = depthOrArraySize;
//         MipLevels = mipLevels;
//         Format = format;
//         SampleDesc.Count = sampleCount;
//         SampleDesc.Quality = sampleQuality;
//         Layout = layout;
//         Flags = flags;
//         SamplerFeedbackMipRegion.Width = samplerFeedbackMipRegionWidth;
//         SamplerFeedbackMipRegion.Height = samplerFeedbackMipRegionHeight;
//         SamplerFeedbackMipRegion.Depth = samplerFeedbackMipRegionDepth;
//     }

//     static inline CD3DX12_RESOURCE_DESC1 Buffer(
//         const D3D12_RESOURCE_ALLOCATION_INFO& resAllocInfo,
//         D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE ) noexcept
//     {
//         return CD3DX12_RESOURCE_DESC1( D3D12_RESOURCE_DIMENSION_BUFFER, resAllocInfo.Alignment, resAllocInfo.SizeInBytes,
//             1, 1, 1, DXGI_FORMAT_UNKNOWN, 1, 0, D3D12_TEXTURE_LAYOUT_ROW_MAJOR, flags, 0, 0, 0 );
//     }
//     static inline CD3DX12_RESOURCE_DESC1 Buffer(
//         UINT64 width,
//         D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
//         UINT64 alignment = 0 ) noexcept
//     {
//         return CD3DX12_RESOURCE_DESC1( D3D12_RESOURCE_DIMENSION_BUFFER, alignment, width, 1, 1, 1,
//             DXGI_FORMAT_UNKNOWN, 1, 0, D3D12_TEXTURE_LAYOUT_ROW_MAJOR, flags, 0, 0, 0 );
//     }
//     static inline CD3DX12_RESOURCE_DESC1 Tex1D(
//         DXGI_FORMAT format,
//         UINT64 width,
//         UINT16 arraySize = 1,
//         UINT16 mipLevels = 0,
//         D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
//         D3D12_TEXTURE_LAYOUT layout = D3D12_TEXTURE_LAYOUT_UNKNOWN,
//         UINT64 alignment = 0 ) noexcept
//     {
//         return CD3DX12_RESOURCE_DESC1( D3D12_RESOURCE_DIMENSION_TEXTURE1D, alignment, width, 1, arraySize,
//             mipLevels, format, 1, 0, layout, flags, 0, 0, 0 );
//     }
//     static inline CD3DX12_RESOURCE_DESC1 Tex2D(
//         DXGI_FORMAT format,
//         UINT64 width,
//         UINT height,
//         UINT16 arraySize = 1,
//         UINT16 mipLevels = 0,
//         UINT sampleCount = 1,
//         UINT sampleQuality = 0,
//         D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
//         D3D12_TEXTURE_LAYOUT layout = D3D12_TEXTURE_LAYOUT_UNKNOWN,
//         UINT64 alignment = 0,
//         UINT samplerFeedbackMipRegionWidth = 0,
//         UINT samplerFeedbackMipRegionHeight = 0,
//         UINT samplerFeedbackMipRegionDepth = 0) noexcept
//     {
//         return CD3DX12_RESOURCE_DESC1( D3D12_RESOURCE_DIMENSION_TEXTURE2D, alignment, width, height, arraySize,
//             mipLevels, format, sampleCount, sampleQuality, layout, flags, samplerFeedbackMipRegionWidth,
//             samplerFeedbackMipRegionHeight, samplerFeedbackMipRegionDepth );
//     }
//     static inline CD3DX12_RESOURCE_DESC1 Tex3D(
//         DXGI_FORMAT format,
//         UINT64 width,
//         UINT height,
//         UINT16 depth,
//         UINT16 mipLevels = 0,
//         D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE,
//         D3D12_TEXTURE_LAYOUT layout = D3D12_TEXTURE_LAYOUT_UNKNOWN,
//         UINT64 alignment = 0 ) noexcept
//     {
//         return CD3DX12_RESOURCE_DESC1( D3D12_RESOURCE_DIMENSION_TEXTURE3D, alignment, width, height, depth,
//             mipLevels, format, 1, 0, layout, flags, 0, 0, 0 );
//     }
//     inline UINT16 Depth() const noexcept
//     { return (Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D ? DepthOrArraySize : 1u); }
//     inline UINT16 ArraySize() const noexcept
//     { return (Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE3D ? DepthOrArraySize : 1u); }
//     inline UINT8 PlaneCount(_In_ ID3D12Device* pDevice) const noexcept
//     { return D3D12GetFormatPlaneCount(pDevice, Format); }
//     inline UINT Subresources(_In_ ID3D12Device* pDevice) const noexcept
//     { return static_cast<UINT>(MipLevels) * ArraySize() * PlaneCount(pDevice); }
//     inline UINT CalcSubresource(UINT MipSlice, UINT ArraySlice, UINT PlaneSlice) noexcept
//     { return D3D12CalcSubresource(MipSlice, ArraySlice, PlaneSlice, MipLevels, ArraySize()); }
// };
// inline bool operator==( const D3D12_RESOURCE_DESC1& l, const D3D12_RESOURCE_DESC1& r ) noexcept
// {
//     return l.Dimension == r.Dimension &&
//         l.Alignment == r.Alignment &&
//         l.Width == r.Width &&
//         l.Height == r.Height &&
//         l.DepthOrArraySize == r.DepthOrArraySize &&
//         l.MipLevels == r.MipLevels &&
//         l.Format == r.Format &&
//         l.SampleDesc.Count == r.SampleDesc.Count &&
//         l.SampleDesc.Quality == r.SampleDesc.Quality &&
//         l.Layout == r.Layout &&
//         l.Flags == r.Flags &&
//         l.SamplerFeedbackMipRegion.Width == r.SamplerFeedbackMipRegion.Width &&
//         l.SamplerFeedbackMipRegion.Height == r.SamplerFeedbackMipRegion.Height &&
//         l.SamplerFeedbackMipRegion.Depth == r.SamplerFeedbackMipRegion.Depth;
// }
// inline bool operator!=( const D3D12_RESOURCE_DESC1& l, const D3D12_RESOURCE_DESC1& r ) noexcept
// { return !( l == r ); }

// //------------------------------------------------------------------------------------------------
// // Fills in the mipmap and alignment values of pDesc when either members are zero
// // Used to replace an implicit field to an explicit (0 mip map = max mip map level)
// // If expansion has occured, returns LclDesc, else returns the original pDesc
// inline const CD3DX12_RESOURCE_DESC1* D3DX12ConditionallyExpandAPIDesc(
//     CD3DX12_RESOURCE_DESC1& LclDesc,
//     const CD3DX12_RESOURCE_DESC1* pDesc)
// {
//     // Expand mip levels:
//     if (pDesc->MipLevels == 0 || pDesc->Alignment == 0)
//     {
//         LclDesc = *pDesc;
//         if (pDesc->MipLevels == 0)
//         {
//             auto MaxMipLevels = [](UINT64 uiMaxDimension) -> UINT16
//             {
//                 UINT16 uiRet = 0;
//                 while (uiMaxDimension > 0)
//                 {
//                     uiRet++;
//                     uiMaxDimension >>= 1;
//                 }
//                 return uiRet;
//             };
//             auto Max = [](UINT64 const & a, UINT64 const & b)
//             {
//                 return (a < b) ? b : a;
//             };

//             LclDesc.MipLevels = MaxMipLevels(
//                 Max(LclDesc.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D ? LclDesc.DepthOrArraySize : 1,
//                     Max(LclDesc.Width, LclDesc.Height)));
//         }
//         if (pDesc->Alignment == 0)
//         {
//             if (pDesc->Layout == D3D12_TEXTURE_LAYOUT_64KB_UNDEFINED_SWIZZLE
//                 || pDesc->Layout == D3D12_TEXTURE_LAYOUT_64KB_STANDARD_SWIZZLE
//                 )
//             {
//                 LclDesc.Alignment = D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT;
//             }
//             else
//             {
//                 LclDesc.Alignment =
//                     (pDesc->SampleDesc.Count > 1 ? D3D12_DEFAULT_MSAA_RESOURCE_PLACEMENT_ALIGNMENT : D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT);
//             }
//         }
//         return &LclDesc;
//     }
//     else
//     {
//         return pDesc;
//     }
// }

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_VIEW_INSTANCING_DESC : public D3D12_VIEW_INSTANCING_DESC
// {
//     CD3DX12_VIEW_INSTANCING_DESC() = default;
//     explicit CD3DX12_VIEW_INSTANCING_DESC( const D3D12_VIEW_INSTANCING_DESC& o ) noexcept :
//         D3D12_VIEW_INSTANCING_DESC( o )
//     {}
//     explicit CD3DX12_VIEW_INSTANCING_DESC( CD3DX12_DEFAULT ) noexcept
//     {
//         ViewInstanceCount = 0;
//         pViewInstanceLocations = nullptr;
//         Flags = D3D12_VIEW_INSTANCING_FLAG_NONE;
//     }
//     explicit CD3DX12_VIEW_INSTANCING_DESC(
//         UINT InViewInstanceCount,
//         const D3D12_VIEW_INSTANCE_LOCATION* InViewInstanceLocations,
//         D3D12_VIEW_INSTANCING_FLAGS InFlags) noexcept
//     {
//         ViewInstanceCount = InViewInstanceCount;
//         pViewInstanceLocations = InViewInstanceLocations;
//         Flags = InFlags;
//     }
// };

// //------------------------------------------------------------------------------------------------
// struct CD3DX12_RT_FORMAT_ARRAY : public D3D12_RT_FORMAT_ARRAY
// {
//     CD3DX12_RT_FORMAT_ARRAY() = default;
//     explicit CD3DX12_RT_FORMAT_ARRAY(const D3D12_RT_FORMAT_ARRAY& o) noexcept
//         : D3D12_RT_FORMAT_ARRAY(o)
//     {}
//     explicit CD3DX12_RT_FORMAT_ARRAY(_In_reads_(NumFormats) const DXGI_FORMAT* pFormats, UINT NumFormats) noexcept
//     {
//         NumRenderTargets = NumFormats;
//         memcpy(RTFormats, pFormats, sizeof(RTFormats));
//         // assumes ARRAY_SIZE(pFormats) == ARRAY_SIZE(RTFormats)
//     }
// };
