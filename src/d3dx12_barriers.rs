use std::{mem::transmute_copy, u32};

use windows::Win32::Graphics::Direct3D12::{
    ID3D12Resource, D3D12_BARRIER_ACCESS, D3D12_BARRIER_GROUP, D3D12_BARRIER_GROUP_0,
    D3D12_BARRIER_LAYOUT, D3D12_BARRIER_SUBRESOURCE_RANGE, D3D12_BARRIER_SYNC,
    D3D12_BARRIER_TYPE_BUFFER, D3D12_BARRIER_TYPE_GLOBAL, D3D12_BARRIER_TYPE_TEXTURE,
    D3D12_BUFFER_BARRIER, D3D12_GLOBAL_BARRIER, D3D12_RESOURCE_ALIASING_BARRIER,
    D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
    D3D12_RESOURCE_BARRIER_FLAGS, D3D12_RESOURCE_BARRIER_FLAG_NONE,
    D3D12_RESOURCE_BARRIER_TYPE_ALIASING, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
    D3D12_RESOURCE_STATES, D3D12_RESOURCE_TRANSITION_BARRIER, D3D12_RESOURCE_UAV_BARRIER,
    D3D12_TEXTURE_BARRIER, D3D12_TEXTURE_BARRIER_FLAGS, D3D12_TEXTURE_BARRIER_FLAG_NONE,
};

#[allow(non_camel_case_types)]
pub trait CD3DX12_RESOURCE_BARRIER {
    fn transition(
        resource: &ID3D12Resource,
        state_before: D3D12_RESOURCE_STATES,
        state_after: D3D12_RESOURCE_STATES,
        subresource: Option<u32>,
        flags: Option<D3D12_RESOURCE_BARRIER_FLAGS>,
    ) -> D3D12_RESOURCE_BARRIER {
        let subresource = subresource.unwrap_or(D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES);
        let flags = flags.unwrap_or(D3D12_RESOURCE_BARRIER_FLAG_NONE);

        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: flags,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: unsafe { std::mem::transmute_copy(resource) },
                    StateBefore: state_before,
                    StateAfter: state_after,
                    Subresource: subresource,
                }),
            },
        }
    }

    fn aliasing(
        resource_before: &ID3D12Resource,
        resource_after: &ID3D12Resource,
    ) -> D3D12_RESOURCE_BARRIER {
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_ALIASING,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Aliasing: std::mem::ManuallyDrop::new(D3D12_RESOURCE_ALIASING_BARRIER {
                    pResourceBefore: unsafe { std::mem::transmute_copy(resource_before) },
                    pResourceAfter: unsafe { std::mem::transmute_copy(resource_after) },
                }),
            },
        }
    }

    fn uav(resource: &ID3D12Resource) -> D3D12_RESOURCE_BARRIER {
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_ALIASING,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                UAV: std::mem::ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                    pResource: unsafe { std::mem::transmute_copy(resource) },
                }),
            },
        }
    }
}

impl CD3DX12_RESOURCE_BARRIER for D3D12_RESOURCE_BARRIER {}

#[allow(non_camel_case_types)]
pub trait CD3DX12_BARRIER_SUBRESOURCE_RANGE {
    fn subresource(subresource: u32) -> D3D12_BARRIER_SUBRESOURCE_RANGE {
        D3D12_BARRIER_SUBRESOURCE_RANGE {
            IndexOrFirstMipLevel: subresource,
            NumMipLevels: 0,
            FirstArraySlice: 0,
            NumArraySlices: 0,
            FirstPlane: 0,
            NumPlanes: 0,
        }
    }
    fn new(
        first_mip_level: u32,
        num_mips: u32,
        first_array_slice: u32,
        num_array_slices: u32,
        first_plane: Option<u32>,
        num_planes: Option<u32>,
    ) -> D3D12_BARRIER_SUBRESOURCE_RANGE {
        let first_plane = first_plane.unwrap_or(0);
        let num_planes = num_planes.unwrap_or(1);

        D3D12_BARRIER_SUBRESOURCE_RANGE {
            IndexOrFirstMipLevel: first_mip_level,
            NumMipLevels: num_mips,
            FirstArraySlice: first_array_slice,
            NumArraySlices: num_array_slices,
            FirstPlane: first_plane,
            NumPlanes: num_planes,
        }
    }
}
impl CD3DX12_BARRIER_SUBRESOURCE_RANGE for D3D12_BARRIER_SUBRESOURCE_RANGE {}

#[allow(non_camel_case_types)]
pub trait CD3DX12_GLOBAL_BARRIER {
    fn new(
        sync_before: D3D12_BARRIER_SYNC,
        sync_after: D3D12_BARRIER_SYNC,
        access_before: D3D12_BARRIER_ACCESS,
        access_after: D3D12_BARRIER_ACCESS,
    ) -> D3D12_GLOBAL_BARRIER {
        D3D12_GLOBAL_BARRIER {
            SyncBefore: sync_before,
            SyncAfter: sync_after,
            AccessBefore: access_before,
            AccessAfter: access_after,
        }
    }
}
impl CD3DX12_GLOBAL_BARRIER for D3D12_GLOBAL_BARRIER {}

#[allow(non_camel_case_types)]
pub trait CD3DX12_BUFFER_BARRIER {
    fn new(
        sync_before: D3D12_BARRIER_SYNC,
        sync_after: D3D12_BARRIER_SYNC,
        access_before: D3D12_BARRIER_ACCESS,
        access_after: D3D12_BARRIER_ACCESS,
        resource: &ID3D12Resource,
    ) -> D3D12_BUFFER_BARRIER {
        D3D12_BUFFER_BARRIER {
            SyncBefore: sync_before,
            SyncAfter: sync_after,
            AccessBefore: access_before,
            AccessAfter: access_after,
            pResource: unsafe { std::mem::transmute_copy(resource) },
            Offset: 0,
            Size: u64::MAX,
        }
    }
}
impl CD3DX12_BUFFER_BARRIER for D3D12_BUFFER_BARRIER {}

#[allow(non_camel_case_types)]
pub trait CD3DX12_TEXTURE_BARRIER {
    fn new(
        sync_before: D3D12_BARRIER_SYNC,
        sync_after: D3D12_BARRIER_SYNC,
        access_before: D3D12_BARRIER_ACCESS,
        access_after: D3D12_BARRIER_ACCESS,
        layout_before: D3D12_BARRIER_LAYOUT,
        layout_after: D3D12_BARRIER_LAYOUT,
        resource: &ID3D12Resource,
        subresources: D3D12_BARRIER_SUBRESOURCE_RANGE,
        flag: Option<D3D12_TEXTURE_BARRIER_FLAGS>,
    ) -> D3D12_TEXTURE_BARRIER {
        let flag = flag.unwrap_or(D3D12_TEXTURE_BARRIER_FLAG_NONE);
        D3D12_TEXTURE_BARRIER {
            SyncBefore: sync_before,
            SyncAfter: sync_after,
            AccessBefore: access_before,
            AccessAfter: access_after,
            LayoutBefore: layout_before,
            LayoutAfter: layout_after,
            pResource: unsafe { transmute_copy(resource) },
            Subresources: subresources,
            Flags: flag,
        }
    }
}
impl CD3DX12_TEXTURE_BARRIER for D3D12_TEXTURE_BARRIER {}

#[allow(non_camel_case_types)]
pub trait CD3DX12_BARRIER_GROUP {
    fn buffer(barriers: &[D3D12_BUFFER_BARRIER]) -> D3D12_BARRIER_GROUP {
        D3D12_BARRIER_GROUP {
            Type: D3D12_BARRIER_TYPE_BUFFER,
            NumBarriers: barriers.len() as u32,
            Anonymous: D3D12_BARRIER_GROUP_0 {
                pBufferBarriers: barriers.as_ptr(),
            },
        }
    }
    fn texture(barriers: &[D3D12_TEXTURE_BARRIER]) -> D3D12_BARRIER_GROUP {
        D3D12_BARRIER_GROUP {
            Type: D3D12_BARRIER_TYPE_TEXTURE,
            NumBarriers: barriers.len() as u32,
            Anonymous: D3D12_BARRIER_GROUP_0 {
                pTextureBarriers: barriers.as_ptr(),
            },
        }
    }
    fn global(barriers: &[D3D12_GLOBAL_BARRIER]) -> D3D12_BARRIER_GROUP {
        D3D12_BARRIER_GROUP {
            Type: D3D12_BARRIER_TYPE_GLOBAL,
            NumBarriers: barriers.len() as u32,
            Anonymous: D3D12_BARRIER_GROUP_0 {
                pGlobalBarriers: barriers.as_ptr(),
            },
        }
    }
}
