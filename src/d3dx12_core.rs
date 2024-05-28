//*********************************************************
//
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License (MIT).
//
//*********************************************************

use std::{
    cmp::max,
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{addr_of_mut, copy_nonoverlapping},
};

use windows::Win32::{
    Foundation::{BOOL, FALSE, RECT, TRUE},
    Graphics::{
        Direct3D::ID3DBlob,
        Direct3D12::*,
        Dxgi::Common::{
            DXGI_FORMAT, DXGI_FORMAT_D16_UNORM, DXGI_FORMAT_D24_UNORM_S8_UINT,
            DXGI_FORMAT_D32_FLOAT, DXGI_FORMAT_D32_FLOAT_S8X24_UINT, DXGI_FORMAT_UNKNOWN,
            DXGI_SAMPLE_DESC,
        },
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
const DEFAULT_STENCIL_OP_DESC1: D3D12_DEPTH_STENCILOP_DESC1 = D3D12_DEPTH_STENCILOP_DESC1 {
    StencilFailOp: D3D12_STENCIL_OP_KEEP,
    StencilDepthFailOp: D3D12_STENCIL_OP_KEEP,
    StencilPassOp: D3D12_STENCIL_OP_KEEP,
    StencilFunc: D3D12_COMPARISON_FUNC_ALWAYS,
    StencilReadMask: D3D12_DEFAULT_STENCIL_READ_MASK as u8,
    StencilWriteMask: D3D12_DEFAULT_STENCIL_WRITE_MASK as u8,
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

//------------------------------------------------------------------------------------------------
// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 606)
#[cfg(feature = "d3d12_sdk_version_gte_606")]
pub trait CD3DX12_DEPTH_STENCIL_DESC2 {
    fn from_D3D12_DEPTH_STENCIL_DESC1(o: &D3D12_DEPTH_STENCIL_DESC1) -> D3D12_DEPTH_STENCIL_DESC2 {
        D3D12_DEPTH_STENCIL_DESC2 {
            DepthEnable: o.DepthEnable,
            DepthWriteMask: o.DepthWriteMask,
            DepthFunc: o.DepthFunc,
            StencilEnable: o.StencilEnable,
            FrontFace: D3D12_DEPTH_STENCILOP_DESC1 {
                StencilFailOp: o.FrontFace.StencilFailOp,
                StencilDepthFailOp: o.FrontFace.StencilDepthFailOp,
                StencilPassOp: o.FrontFace.StencilPassOp,
                StencilFunc: o.FrontFace.StencilFunc,
                StencilReadMask: o.StencilReadMask,
                StencilWriteMask: o.StencilWriteMask,
            },
            BackFace: D3D12_DEPTH_STENCILOP_DESC1 {
                StencilFailOp: o.BackFace.StencilFailOp,
                StencilDepthFailOp: o.BackFace.StencilDepthFailOp,
                StencilPassOp: o.BackFace.StencilPassOp,
                StencilFunc: o.BackFace.StencilFunc,
                StencilReadMask: o.StencilReadMask,
                StencilWriteMask: o.StencilWriteMask,
            },
            DepthBoundsTestEnable: o.DepthBoundsTestEnable,
        }
    }
    fn from_D3D12_DEPTH_STENCIL_DESC(o: &D3D12_DEPTH_STENCIL_DESC) -> D3D12_DEPTH_STENCIL_DESC2 {
        D3D12_DEPTH_STENCIL_DESC2 {
            DepthEnable: o.DepthEnable,
            DepthWriteMask: o.DepthWriteMask,
            DepthFunc: o.DepthFunc,
            StencilEnable: o.StencilEnable,
            FrontFace: D3D12_DEPTH_STENCILOP_DESC1 {
                StencilFailOp: o.FrontFace.StencilFailOp,
                StencilDepthFailOp: o.FrontFace.StencilDepthFailOp,
                StencilPassOp: o.FrontFace.StencilPassOp,
                StencilFunc: o.FrontFace.StencilFunc,
                StencilReadMask: o.StencilReadMask,
                StencilWriteMask: o.StencilWriteMask,
            },
            BackFace: D3D12_DEPTH_STENCILOP_DESC1 {
                StencilFailOp: o.BackFace.StencilFailOp,
                StencilDepthFailOp: o.BackFace.StencilDepthFailOp,
                StencilPassOp: o.BackFace.StencilPassOp,
                StencilFunc: o.BackFace.StencilFunc,
                StencilReadMask: o.StencilReadMask,
                StencilWriteMask: o.StencilWriteMask,
            },
            DepthBoundsTestEnable: FALSE,
        }
    }
    fn default() -> D3D12_DEPTH_STENCIL_DESC2 {
        D3D12_DEPTH_STENCIL_DESC2 {
            DepthEnable: TRUE,
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_LESS,
            StencilEnable: FALSE,
            FrontFace: DEFAULT_STENCIL_OP_DESC1,
            BackFace: DEFAULT_STENCIL_OP_DESC1,
            DepthBoundsTestEnable: FALSE,
        }
    }
    fn new(
        depth_enable: BOOL,
        depth_write_mask: D3D12_DEPTH_WRITE_MASK,
        depth_func: D3D12_COMPARISON_FUNC,
        stencil_enable: BOOL,
        front_stencil_fail_op: D3D12_STENCIL_OP,
        front_stencil_depth_fail_op: D3D12_STENCIL_OP,
        front_stencil_pass_op: D3D12_STENCIL_OP,
        front_stencil_func: D3D12_COMPARISON_FUNC,
        front_stencil_read_mask: u8,
        front_stencil_write_mask: u8,
        back_stencil_fail_op: D3D12_STENCIL_OP,
        back_stencil_depth_fail_op: D3D12_STENCIL_OP,
        back_stencil_pass_op: D3D12_STENCIL_OP,
        back_stencil_func: D3D12_COMPARISON_FUNC,
        back_stencil_read_mask: u8,
        back_stencil_write_mask: u8,
        depth_bounds_test_enable: BOOL,
    ) -> D3D12_DEPTH_STENCIL_DESC2 {
        D3D12_DEPTH_STENCIL_DESC2 {
            DepthEnable: depth_enable,
            DepthWriteMask: depth_write_mask,
            DepthFunc: depth_func,
            StencilEnable: stencil_enable,
            FrontFace: D3D12_DEPTH_STENCILOP_DESC1 {
                StencilFailOp: front_stencil_fail_op,
                StencilDepthFailOp: front_stencil_depth_fail_op,
                StencilPassOp: front_stencil_pass_op,
                StencilFunc: front_stencil_func,
                StencilReadMask: front_stencil_read_mask,
                StencilWriteMask: front_stencil_write_mask,
            },
            BackFace: D3D12_DEPTH_STENCILOP_DESC1 {
                StencilFailOp: back_stencil_fail_op,
                StencilDepthFailOp: back_stencil_depth_fail_op,
                StencilPassOp: back_stencil_pass_op,
                StencilFunc: back_stencil_func,
                StencilReadMask: back_stencil_read_mask,
                StencilWriteMask: back_stencil_write_mask,
            },
            DepthBoundsTestEnable: depth_bounds_test_enable,
        }
    }
}

#[cfg(feature = "d3d12_sdk_version_gte_606")]
impl CD3DX12_DEPTH_STENCIL_DESC2 for D3D12_DEPTH_STENCIL_DESC2 {}

//------------------------------------------------------------------------------------------------
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

//------------------------------------------------------------------------------------------------
// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 608)
#[cfg(feature = "d3d12_sdk_version_gte_608")]
pub trait CD3DX12_RASTERIZER_DESC1 {
    fn from_D3D12_RASTERIZER_DESC(o: &D3D12_RASTERIZER_DESC) -> D3D12_RASTERIZER_DESC1 {
        D3D12_RASTERIZER_DESC1 {
            FillMode: o.FillMode,
            CullMode: o.CullMode,
            FrontCounterClockwise: o.FrontCounterClockwise,
            DepthBias: o.DepthBias as f32,
            DepthBiasClamp: o.DepthBiasClamp,
            SlopeScaledDepthBias: o.SlopeScaledDepthBias,
            DepthClipEnable: o.DepthClipEnable,
            MultisampleEnable: o.MultisampleEnable,
            AntialiasedLineEnable: o.AntialiasedLineEnable,
            ForcedSampleCount: o.ForcedSampleCount,
            ConservativeRaster: o.ConservativeRaster,
        }
    }
    fn default() -> D3D12_RASTERIZER_DESC1 {
        D3D12_RASTERIZER_DESC1 {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            FrontCounterClockwise: FALSE,
            DepthBias: D3D12_DEFAULT_DEPTH_BIAS as f32,
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
        depth_bias: f32,
        depth_bias_clamp: f32,
        slope_scaled_depth_bias: f32,
        depth_clip_enable: BOOL,
        multisample_enable: BOOL,
        antialiased_line_enable: BOOL,
        forced_sample_count: u32,
        conservative_raster: D3D12_CONSERVATIVE_RASTERIZATION_MODE,
    ) -> D3D12_RASTERIZER_DESC1 {
        D3D12_RASTERIZER_DESC1 {
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
    fn to_D3D12_RASTERIZER_DESC(&self) -> D3D12_RASTERIZER_DESC;
}
#[cfg(feature = "d3d12_sdk_version_gte_608")]
impl CD3DX12_RASTERIZER_DESC1 for D3D12_RASTERIZER_DESC1 {
    fn to_D3D12_RASTERIZER_DESC(&self) -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: self.FillMode,
            CullMode: self.CullMode,
            FrontCounterClockwise: self.FrontCounterClockwise,
            DepthBias: self.DepthBias as i32,
            DepthBiasClamp: self.DepthBiasClamp,
            SlopeScaledDepthBias: self.SlopeScaledDepthBias,
            DepthClipEnable: self.DepthClipEnable,
            MultisampleEnable: self.MultisampleEnable,
            AntialiasedLineEnable: self.AntialiasedLineEnable,
            ForcedSampleCount: self.ForcedSampleCount,
            ConservativeRaster: self.ConservativeRaster,
        }
    }
}

//------------------------------------------------------------------------------------------------
// #if defined(D3D12_SDK_VERSION) && (D3D12_SDK_VERSION >= 610)
pub trait CD3DX12_RASTERIZER_DESC2 {
    fn from_D3D12_RASTERIZER_DESC1(o: &D3D12_RASTERIZER_DESC1) -> D3D12_RASTERIZER_DESC2 {
        let line_rasterization_mode = if o.MultisampleEnable.as_bool() {
            D3D12_LINE_RASTERIZATION_MODE_QUADRILATERAL_WIDE
        } else if o.AntialiasedLineEnable.as_bool() {
            D3D12_LINE_RASTERIZATION_MODE_ALPHA_ANTIALIASED
        } else {
            D3D12_LINE_RASTERIZATION_MODE_ALIASED
        };
        D3D12_RASTERIZER_DESC2 {
            FillMode: o.FillMode,
            CullMode: o.CullMode,
            FrontCounterClockwise: o.FrontCounterClockwise,
            DepthBias: o.DepthBias,
            DepthBiasClamp: o.DepthBiasClamp,
            SlopeScaledDepthBias: o.SlopeScaledDepthBias,
            DepthClipEnable: o.DepthClipEnable,
            LineRasterizationMode: line_rasterization_mode,
            ForcedSampleCount: o.ForcedSampleCount,
            ConservativeRaster: o.ConservativeRaster,
        }
    }
    fn from_D3D12_RASTERIZER_DESC(o: &D3D12_RASTERIZER_DESC) -> D3D12_RASTERIZER_DESC2 {
        D3D12_RASTERIZER_DESC2::from_D3D12_RASTERIZER_DESC1(
            &D3D12_RASTERIZER_DESC1::from_D3D12_RASTERIZER_DESC(o),
        )
    }
    fn default() -> D3D12_RASTERIZER_DESC2 {
        D3D12_RASTERIZER_DESC2 {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            FrontCounterClockwise: FALSE,
            DepthBias: D3D12_DEFAULT_DEPTH_BIAS as f32,
            DepthBiasClamp: D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
            SlopeScaledDepthBias: D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
            DepthClipEnable: TRUE,
            LineRasterizationMode: D3D12_LINE_RASTERIZATION_MODE_ALIASED,
            ForcedSampleCount: 0,
            ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
        }
    }
    fn new(
        fill_mode: D3D12_FILL_MODE,
        cull_mode: D3D12_CULL_MODE,
        front_counter_clockwise: BOOL,
        depth_bias: f32,
        depth_bias_clamp: f32,
        slope_scaled_depth_bias: f32,
        depth_clip_enable: BOOL,
        line_rasterization_mode: D3D12_LINE_RASTERIZATION_MODE,
        forced_sample_count: u32,
        conservative_raster: D3D12_CONSERVATIVE_RASTERIZATION_MODE,
    ) -> D3D12_RASTERIZER_DESC2 {
        D3D12_RASTERIZER_DESC2 {
            FillMode: fill_mode,
            CullMode: cull_mode,
            FrontCounterClockwise: front_counter_clockwise,
            DepthBias: depth_bias,
            DepthBiasClamp: depth_bias_clamp,
            SlopeScaledDepthBias: slope_scaled_depth_bias,
            DepthClipEnable: depth_clip_enable,
            LineRasterizationMode: line_rasterization_mode,
            ForcedSampleCount: forced_sample_count,
            ConservativeRaster: conservative_raster,
        }
    }
    fn to_D3D12_RASTERIZER_DESC1(&self) -> D3D12_RASTERIZER_DESC1;

    //     operator D3D12_RASTERIZER_DESC() const noexcept
    //     {
    //         return (D3D12_RASTERIZER_DESC)CD3DX12_RASTERIZER_DESC1((D3D12_RASTERIZER_DESC1)*this);
    //     }
}
impl CD3DX12_RASTERIZER_DESC2 for D3D12_RASTERIZER_DESC2 {
    fn to_D3D12_RASTERIZER_DESC1(&self) -> D3D12_RASTERIZER_DESC1 {
        let (antialiased_line_enable, multisample_enable) =
            if self.LineRasterizationMode == D3D12_LINE_RASTERIZATION_MODE_ALPHA_ANTIALIASED {
                (TRUE, FALSE)
            } else if self.LineRasterizationMode != D3D12_LINE_RASTERIZATION_MODE_ALIASED {
                (FALSE, TRUE)
            } else {
                (FALSE, FALSE)
            };
        D3D12_RASTERIZER_DESC1 {
            FillMode: self.FillMode,
            CullMode: self.CullMode,
            FrontCounterClockwise: self.FrontCounterClockwise,
            DepthBias: self.DepthBias,
            DepthBiasClamp: self.DepthBiasClamp,
            SlopeScaledDepthBias: self.SlopeScaledDepthBias,
            DepthClipEnable: self.DepthClipEnable,
            MultisampleEnable: multisample_enable,
            AntialiasedLineEnable: antialiased_line_enable,
            ForcedSampleCount: self.ForcedSampleCount,
            ConservativeRaster: self.ConservativeRaster,
        }
    }
}
// #endif // D3D12_SDK_VERSION >= 610

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_RESOURCE_ALLOCATION_INFO {
    fn new(size: u64, alignment: u64) -> D3D12_RESOURCE_ALLOCATION_INFO {
        D3D12_RESOURCE_ALLOCATION_INFO {
            SizeInBytes: size,
            Alignment: alignment,
        }
    }
}
impl CD3DX12_RESOURCE_ALLOCATION_INFO for D3D12_RESOURCE_ALLOCATION_INFO {}

//------------------------------------------------------------------------------------------------
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

//------------------------------------------------------------------------------------------------

pub trait CD3DX12_HEAP_DESC {
    fn new_with_properties(
        size: u64,
        properties: D3D12_HEAP_PROPERTIES,
        alignment: Option<u64>,
        flags: Option<D3D12_HEAP_FLAGS>,
    ) -> D3D12_HEAP_DESC {
        let alignment = alignment.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_HEAP_FLAG_NONE);
        D3D12_HEAP_DESC {
            SizeInBytes: size,
            Properties: properties,
            Alignment: alignment,
            Flags: flags,
        }
    }
    fn new_with_type(
        size: u64,
        r#type: D3D12_HEAP_TYPE,
        alignment: Option<u64>,
        flags: Option<D3D12_HEAP_FLAGS>,
    ) -> D3D12_HEAP_DESC {
        let alignment = alignment.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_HEAP_FLAG_NONE);
        D3D12_HEAP_DESC {
            SizeInBytes: size,
            Properties: D3D12_HEAP_PROPERTIES::new_with_type(r#type, None, None),
            Alignment: alignment,
            Flags: flags,
        }
    }
    fn new_with_cpu_page_property(
        size: u64,
        cpu_page_property: D3D12_CPU_PAGE_PROPERTY,
        memory_pool_preference: D3D12_MEMORY_POOL,
        alignment: Option<u64>,
        flags: Option<D3D12_HEAP_FLAGS>,
    ) -> D3D12_HEAP_DESC {
        let alignment = alignment.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_HEAP_FLAG_NONE);
        D3D12_HEAP_DESC {
            SizeInBytes: size,
            Properties: D3D12_HEAP_PROPERTIES::new_custom(
                cpu_page_property,
                memory_pool_preference,
                None,
                None,
            ),
            Alignment: alignment,
            Flags: flags,
        }
    }
    fn new_with_res_alloc_info(
        res_alloc_info: D3D12_RESOURCE_ALLOCATION_INFO,
        properties: D3D12_HEAP_PROPERTIES,
        flags: Option<D3D12_HEAP_FLAGS>,
    ) -> D3D12_HEAP_DESC {
        let flags = flags.unwrap_or(D3D12_HEAP_FLAG_NONE);
        D3D12_HEAP_DESC {
            SizeInBytes: res_alloc_info.SizeInBytes,
            Properties: properties,
            Alignment: res_alloc_info.Alignment,
            Flags: flags,
        }
    }
    fn new_with_res_and_type(
        res_alloc_info: D3D12_RESOURCE_ALLOCATION_INFO,
        r#type: D3D12_HEAP_TYPE,
        flags: Option<D3D12_HEAP_FLAGS>,
    ) -> D3D12_HEAP_DESC {
        let flags = flags.unwrap_or(D3D12_HEAP_FLAG_NONE);
        D3D12_HEAP_DESC {
            SizeInBytes: res_alloc_info.SizeInBytes,
            Properties: D3D12_HEAP_PROPERTIES::new_with_type(r#type, None, None),
            Alignment: res_alloc_info.Alignment,
            Flags: flags,
        }
    }
    fn new_with_res_alloc_info_and_cpu_page_property(
        res_alloc_info: D3D12_RESOURCE_ALLOCATION_INFO,
        cpu_page_property: D3D12_CPU_PAGE_PROPERTY,
        memory_pool_preference: D3D12_MEMORY_POOL,
        flags: Option<D3D12_HEAP_FLAGS>,
    ) -> D3D12_HEAP_DESC {
        let flags = flags.unwrap_or(D3D12_HEAP_FLAG_NONE);
        D3D12_HEAP_DESC {
            SizeInBytes: res_alloc_info.SizeInBytes,
            Properties: D3D12_HEAP_PROPERTIES::new_custom(
                cpu_page_property,
                memory_pool_preference,
                None,
                None,
            ),
            Alignment: res_alloc_info.Alignment,
            Flags: flags,
        }
    }
    fn is_cpu_accessible(&self) -> bool;
}

impl CD3DX12_HEAP_DESC for D3D12_HEAP_DESC {
    fn is_cpu_accessible(&self) -> bool {
        self.Properties.is_cpu_accessible()
    }
}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_CLEAR_VALUE {
    fn new_color(format: DXGI_FORMAT, color: [f32; 4]) -> D3D12_CLEAR_VALUE {
        D3D12_CLEAR_VALUE {
            Format: format,
            Anonymous: D3D12_CLEAR_VALUE_0 { Color: color },
        }
    }
    fn new_depth_stencil(format: DXGI_FORMAT, depth: f32, stencil: u8) -> D3D12_CLEAR_VALUE {
        let mut clear_value = D3D12_CLEAR_VALUE {
            Format: format,
            Anonymous: D3D12_CLEAR_VALUE_0 {
                // TODO: This was in the original code. Is it necessary?
                Color: unsafe { core::mem::zeroed() },
            },
        };
        clear_value.Anonymous.DepthStencil = D3D12_DEPTH_STENCIL_VALUE {
            // TODO: This was in the original code. Is it necessary?
            // Use memcpy to preserve NAN values
            // memcpy( &DepthStencil.Depth, &depth, sizeof( depth ) );
            Depth: depth,
            Stencil: stencil,
        };
        clear_value
    }
}
impl CD3DX12_CLEAR_VALUE for D3D12_CLEAR_VALUE {}
pub struct D3DX12_CLEAR_VALUE_TYPE(pub D3D12_CLEAR_VALUE);
impl PartialEq for D3DX12_CLEAR_VALUE_TYPE {
    fn eq(&self, other: &Self) -> bool {
        if self.0.Format != other.0.Format {
            return false;
        }
        if self.0.Format == DXGI_FORMAT_D24_UNORM_S8_UINT
            || self.0.Format == DXGI_FORMAT_D16_UNORM
            || self.0.Format == DXGI_FORMAT_D32_FLOAT
            || self.0.Format == DXGI_FORMAT_D32_FLOAT_S8X24_UINT
        {
            return self.0.Anonymous.DepthStencil.Depth == other.0.Anonymous.DepthStencil.Depth
                && self.0.Anonymous.DepthStencil.Stencil == other.0.Anonymous.DepthStencil.Stencil;
        } else {
            return self.0.Anonymous.Color == other.0.Anonymous.Color;
        }
    }
}

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

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_RANGE_UINT64 {
    fn new(begin: u64, end: u64) -> D3D12_RANGE_UINT64 {
        D3D12_RANGE_UINT64 {
            Begin: begin,
            End: end,
        }
    }
}
impl CD3DX12_RANGE_UINT64 for D3D12_RANGE_UINT64 {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_SUBRESOURCE_RANGE_UINT64 {
    fn new_with_range(
        subresource: u32,
        range: D3D12_RANGE_UINT64,
    ) -> D3D12_SUBRESOURCE_RANGE_UINT64 {
        D3D12_SUBRESOURCE_RANGE_UINT64 {
            Subresource: subresource,
            Range: range,
        }
    }
    fn new(subresource: u32, begin: u64, end: u64) -> D3D12_SUBRESOURCE_RANGE_UINT64 {
        D3D12_SUBRESOURCE_RANGE_UINT64 {
            Subresource: subresource,
            Range: D3D12_RANGE_UINT64 {
                Begin: begin,
                End: end,
            },
        }
    }
}

//------------------------------------------------------------------------------------------------
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

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_TILED_RESOURCE_COORDINATE {
    fn new(x: u32, y: u32, z: u32, subresource: u32) -> D3D12_TILED_RESOURCE_COORDINATE {
        D3D12_TILED_RESOURCE_COORDINATE {
            X: x,
            Y: y,
            Z: z,
            Subresource: subresource,
        }
    }
}
impl CD3DX12_TILED_RESOURCE_COORDINATE for D3D12_TILED_RESOURCE_COORDINATE {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_TILE_REGION_SIZE {
    fn new(
        num_tiles: u32,
        use_box: BOOL,
        width: u32,
        height: u16,
        depth: u16,
    ) -> D3D12_TILE_REGION_SIZE {
        D3D12_TILE_REGION_SIZE {
            NumTiles: num_tiles,
            UseBox: use_box,
            Width: width,
            Height: height,
            Depth: depth,
        }
    }
}
impl CD3DX12_TILE_REGION_SIZE for D3D12_TILE_REGION_SIZE {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_SUBRESOURCE_TILING {
    fn new(
        width_in_tiles: u32,
        height_in_tiles: u16,
        depth_in_tiles: u16,
        start_tile_index_in_overall_resource: u32,
    ) -> D3D12_SUBRESOURCE_TILING {
        D3D12_SUBRESOURCE_TILING {
            WidthInTiles: width_in_tiles,
            HeightInTiles: height_in_tiles,
            DepthInTiles: depth_in_tiles,
            StartTileIndexInOverallResource: start_tile_index_in_overall_resource,
        }
    }
}
impl CD3DX12_SUBRESOURCE_TILING for D3D12_SUBRESOURCE_TILING {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_TILE_SHAPE {
    fn new(width_in_texels: u32, height_in_texels: u32, depth_in_texels: u32) -> D3D12_TILE_SHAPE {
        D3D12_TILE_SHAPE {
            WidthInTexels: width_in_texels,
            HeightInTexels: height_in_texels,
            DepthInTexels: depth_in_texels,
        }
    }
}
impl CD3DX12_TILE_SHAPE for D3D12_TILE_SHAPE {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_PACKED_MIP_INFO {
    fn new(
        num_standard_mips: u8,
        num_packed_mips: u8,
        num_tiles_for_packed_mips: u32,
        start_tile_index_in_overall_resource: u32,
    ) -> D3D12_PACKED_MIP_INFO {
        D3D12_PACKED_MIP_INFO {
            NumStandardMips: num_standard_mips,
            NumPackedMips: num_packed_mips,
            NumTilesForPackedMips: num_tiles_for_packed_mips,
            StartTileIndexInOverallResource: start_tile_index_in_overall_resource,
        }
    }
}
impl CD3DX12_PACKED_MIP_INFO for D3D12_PACKED_MIP_INFO {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_SUBRESOURCE_FOOTPRINT {
    fn new(
        format: DXGI_FORMAT,
        width: u32,
        height: u32,
        depth: u32,
        row_pitch: u32,
    ) -> D3D12_SUBRESOURCE_FOOTPRINT {
        D3D12_SUBRESOURCE_FOOTPRINT {
            Format: format,
            Width: width,
            Height: height,
            Depth: depth,
            RowPitch: row_pitch,
        }
    }
    fn from_res_desc(
        res_desc: &D3D12_RESOURCE_DESC,
        row_pitch: u32,
    ) -> D3D12_SUBRESOURCE_FOOTPRINT {
        D3D12_SUBRESOURCE_FOOTPRINT {
            Format: res_desc.Format,
            Width: res_desc.Width as u32,
            Height: res_desc.Height,
            Depth: if res_desc.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D {
                res_desc.DepthOrArraySize as u32
            } else {
                1
            },
            RowPitch: row_pitch,
        }
    }
}
impl CD3DX12_SUBRESOURCE_FOOTPRINT for D3D12_SUBRESOURCE_FOOTPRINT {}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_TEXTURE_COPY_LOCATION {
    fn new(resource: &ID3D12Resource) -> D3D12_TEXTURE_COPY_LOCATION {
        D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(resource) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0::default(),
        }
    }
    fn new_with_placed_footprint(
        resource: &ID3D12Resource,
        footprint: &D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
    ) -> D3D12_TEXTURE_COPY_LOCATION {
        D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(resource) },
            Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                PlacedFootprint: *footprint,
            },
        }
    }

    fn new_with_subresource_index(
        resource: &ID3D12Resource,
        subresource_index: u32,
    ) -> D3D12_TEXTURE_COPY_LOCATION {
        D3D12_TEXTURE_COPY_LOCATION {
            pResource: unsafe { std::mem::transmute_copy(resource) },
            Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
            Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                SubresourceIndex: subresource_index,
            },
        }
    }
}
impl CD3DX12_TEXTURE_COPY_LOCATION for D3D12_TEXTURE_COPY_LOCATION {}

//------------------------------------------------------------------------------------------------
pub const fn D3D12CalcSubresource(
    mip_slice: u32,
    array_slice: u32,
    plane_slice: u32,
    mip_levels: u32,
    array_size: u32,
) -> u32 {
    mip_slice + array_slice * mip_levels + plane_slice * mip_levels * array_size
}

//------------------------------------------------------------------------------------------------
#[inline]
pub fn D3D12GetFormatPlaneCount(device: &ID3D12Device, format: DXGI_FORMAT) -> u8 {
    let mut format_info = D3D12_FEATURE_DATA_FORMAT_INFO {
        Format: format,
        PlaneCount: 0,
    };
    if unsafe {
        device
            .CheckFeatureSupport(
                D3D12_FEATURE_FORMAT_INFO,
                &mut format_info as *mut _ as *mut _,
                std::mem::size_of_val(&format_info) as u32,
            )
            .is_err()
    } {
        0
    } else {
        format_info.PlaneCount
    }
}

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
    fn buffer(
        width: u64,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC {
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let alignment = alignment.unwrap_or(0);
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

    fn buffer_with_resource_allocation_info(
        res_alloc_info: D3D12_RESOURCE_ALLOCATION_INFO,
        flags: Option<D3D12_RESOURCE_FLAGS>,
    ) -> D3D12_RESOURCE_DESC {
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        Self::new(
            D3D12_RESOURCE_DIMENSION_BUFFER,
            res_alloc_info.Alignment,
            res_alloc_info.SizeInBytes,
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

    fn tex_1d(
        format: DXGI_FORMAT,
        width: u64,
        array_size: Option<u16>,
        mip_levels: Option<u16>,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        layout: Option<D3D12_TEXTURE_LAYOUT>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC {
        let array_size = array_size.unwrap_or(1);
        let mip_levels = mip_levels.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let layout = layout.unwrap_or(D3D12_TEXTURE_LAYOUT_UNKNOWN);
        let alignment = alignment.unwrap_or(0);
        Self::new(
            D3D12_RESOURCE_DIMENSION_TEXTURE1D,
            alignment,
            width,
            1,
            array_size,
            mip_levels,
            format,
            1,
            0,
            layout,
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
    fn tex_3d(
        format: DXGI_FORMAT,
        width: u64,
        height: u32,
        depth: u16,
        mip_levels: Option<u16>,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        layout: Option<D3D12_TEXTURE_LAYOUT>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC {
        let mip_levels = mip_levels.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let layout = layout.unwrap_or(D3D12_TEXTURE_LAYOUT_UNKNOWN);
        let alignment = alignment.unwrap_or(0);
        Self::new(
            D3D12_RESOURCE_DIMENSION_TEXTURE3D,
            alignment,
            width,
            height,
            depth,
            mip_levels,
            format,
            1,
            0,
            layout,
            flags,
        )
    }
    fn depth(&self) -> u16;
    fn array_size(&self) -> u16;
    fn plane_count(&self, device: &ID3D12Device) -> u8;
    fn subresources(&self, device: &ID3D12Device) -> u32;
    fn calc_subresource(&self, mip_slice: u32, array_slice: u32, plane_slice: u32) -> u32;
}
impl CD3DX12_RESOURCE_DESC for D3D12_RESOURCE_DESC {
    #[inline]
    fn depth(&self) -> u16 {
        if self.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D {
            self.DepthOrArraySize as u16
        } else {
            1
        }
    }
    #[inline]
    fn array_size(&self) -> u16 {
        if self.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE3D {
            self.DepthOrArraySize
        } else {
            1
        }
    }
    #[inline]
    fn plane_count(&self, device: &ID3D12Device) -> u8 {
        D3D12GetFormatPlaneCount(device, self.Format)
    }
    #[inline]
    fn subresources(&self, device: &ID3D12Device) -> u32 {
        self.MipLevels as u32 * self.array_size() as u32 * self.plane_count(device) as u32
    }
    #[inline]
    fn calc_subresource(&self, mip_slice: u32, array_slice: u32, plane_slice: u32) -> u32 {
        D3D12CalcSubresource(
            mip_slice,
            array_slice,
            plane_slice,
            self.MipLevels as u32,
            self.array_size() as u32,
        )
    }
}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_RESOURCE_DESC1 {
    fn from_D3D12_RESOURCE_DESC(o: &D3D12_RESOURCE_DESC) -> D3D12_RESOURCE_DESC1 {
        D3D12_RESOURCE_DESC1 {
            Dimension: o.Dimension,
            Alignment: o.Alignment,
            Width: o.Width,
            Height: o.Height,
            DepthOrArraySize: o.DepthOrArraySize,
            MipLevels: o.MipLevels,
            Format: o.Format,
            SampleDesc: o.SampleDesc,
            Layout: o.Layout,
            Flags: o.Flags,
            SamplerFeedbackMipRegion: D3D12_MIP_REGION {
                Width: 0,
                Height: 0,
                Depth: 0,
            },
        }
    }
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
        sampler_feedback_mip_region_width: Option<u32>,
        sampler_feedback_mip_region_height: Option<u32>,
        sampler_feedback_mip_region_depth: Option<u32>,
    ) -> D3D12_RESOURCE_DESC1 {
        let sampler_feedback_mip_region_width = sampler_feedback_mip_region_width.unwrap_or(0);
        let sampler_feedback_mip_region_height = sampler_feedback_mip_region_height.unwrap_or(0);
        let sampler_feedback_mip_region_depth = sampler_feedback_mip_region_depth.unwrap_or(0);
        D3D12_RESOURCE_DESC1 {
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
            SamplerFeedbackMipRegion: D3D12_MIP_REGION {
                Width: sampler_feedback_mip_region_width,
                Height: sampler_feedback_mip_region_height,
                Depth: sampler_feedback_mip_region_depth,
            },
        }
    }
    fn buffer_with_resource_allocation_info(
        res_alloc_info: &D3D12_RESOURCE_ALLOCATION_INFO,
        flags: Option<D3D12_RESOURCE_FLAGS>,
    ) -> D3D12_RESOURCE_DESC1 {
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        Self::new(
            D3D12_RESOURCE_DIMENSION_BUFFER,
            res_alloc_info.Alignment,
            res_alloc_info.SizeInBytes,
            1,
            1,
            1,
            DXGI_FORMAT_UNKNOWN,
            1,
            0,
            D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            flags,
            Some(0),
            Some(0),
            Some(0),
        )
    }
    fn buffer(
        width: u64,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC1 {
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let alignment = alignment.unwrap_or(0);
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
            Some(0),
            Some(0),
            Some(0),
        )
    }
    fn tex_1d(
        format: DXGI_FORMAT,
        width: u64,
        array_size: Option<u16>,
        mip_levels: Option<u16>,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        layout: Option<D3D12_TEXTURE_LAYOUT>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC1 {
        let array_size = array_size.unwrap_or(1);
        let mip_levels = mip_levels.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let layout = layout.unwrap_or(D3D12_TEXTURE_LAYOUT_UNKNOWN);
        let alignment = alignment.unwrap_or(0);
        Self::new(
            D3D12_RESOURCE_DIMENSION_TEXTURE1D,
            alignment,
            width,
            1,
            array_size,
            mip_levels,
            format,
            1,
            0,
            layout,
            flags,
            Some(0),
            Some(0),
            Some(0),
        )
    }
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
        sampler_feedback_mip_region_width: Option<u32>,
        sampler_feedback_mip_region_height: Option<u32>,
        sampler_feedback_mip_region_depth: Option<u32>,
    ) -> D3D12_RESOURCE_DESC1 {
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
            sampler_feedback_mip_region_width,
            sampler_feedback_mip_region_height,
            sampler_feedback_mip_region_depth,
        )
    }
    fn tex_3d(
        format: DXGI_FORMAT,
        width: u64,
        height: u32,
        depth: u16,
        mip_levels: Option<u16>,
        flags: Option<D3D12_RESOURCE_FLAGS>,
        layout: Option<D3D12_TEXTURE_LAYOUT>,
        alignment: Option<u64>,
    ) -> D3D12_RESOURCE_DESC1 {
        let mip_levels = mip_levels.unwrap_or(0);
        let flags = flags.unwrap_or(D3D12_RESOURCE_FLAG_NONE);
        let layout = layout.unwrap_or(D3D12_TEXTURE_LAYOUT_UNKNOWN);
        let alignment = alignment.unwrap_or(0);
        Self::new(
            D3D12_RESOURCE_DIMENSION_TEXTURE3D,
            alignment,
            width,
            height,
            depth,
            mip_levels,
            format,
            1,
            0,
            layout,
            flags,
            Some(0),
            Some(0),
            Some(0),
        )
    }
    fn depth(&self) -> u16;
    fn array_size(&self) -> u16;
    fn plane_count(&self, device: &ID3D12Device) -> u8;
    fn subresources(&self, device: &ID3D12Device) -> u32;
    fn calc_subresource(&self, mip_slice: u32, array_slice: u32, plane_slice: u32) -> u32;
}
impl CD3DX12_RESOURCE_DESC1 for D3D12_RESOURCE_DESC1 {
    #[inline]
    fn depth(&self) -> u16 {
        if self.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D {
            self.DepthOrArraySize
        } else {
            1
        }
    }
    #[inline]
    fn array_size(&self) -> u16 {
        if self.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE3D {
            self.DepthOrArraySize
        } else {
            1
        }
    }
    #[inline]
    fn plane_count(&self, device: &ID3D12Device) -> u8 {
        D3D12GetFormatPlaneCount(device, self.Format)
    }
    #[inline]
    fn subresources(&self, device: &ID3D12Device) -> u32 {
        self.MipLevels as u32 * self.array_size() as u32 * self.plane_count(device) as u32
    }
    #[inline]
    fn calc_subresource(&self, mip_slice: u32, array_slice: u32, plane_slice: u32) -> u32 {
        D3D12CalcSubresource(
            mip_slice,
            array_slice,
            plane_slice,
            self.MipLevels as u32,
            self.array_size() as u32,
        )
    }
}

//------------------------------------------------------------------------------------------------
// Fills in the mipmap and alignment values of pDesc when either members are zero
// Used to replace an implicit field to an explicit (0 mip map = max mip map level)
// If expansion has occured, returns LclDesc, else returns the original pDesc
#[inline]
pub fn D3DX12ConditionallyExpandAPIDesc<'a>(
    lcl_desc: &'a mut D3D12_RESOURCE_DESC1,
    p_desc: &'a D3D12_RESOURCE_DESC1,
) -> &'a D3D12_RESOURCE_DESC1 {
    // Expand mip levels:
    if p_desc.MipLevels == 0 || p_desc.Alignment == 0 {
        *lcl_desc = *p_desc;
        if p_desc.MipLevels == 0 {
            let max_mip_levels = |ui_max_dimension: u64| -> u16 {
                let mut ui_ret = 0;
                let mut ui_max_dimension = ui_max_dimension;
                while ui_max_dimension > 0 {
                    ui_ret += 1;
                    ui_max_dimension >>= 1;
                }
                ui_ret
            };
            // let max = |a: u64, b: u64| -> u64 {
            //     if a < b {
            //         b
            //     } else {
            //         a
            //     }
            // };

            lcl_desc.MipLevels = max_mip_levels(max(
                if lcl_desc.Dimension == D3D12_RESOURCE_DIMENSION_TEXTURE3D {
                    lcl_desc.DepthOrArraySize as u64
                } else {
                    1
                },
                max(lcl_desc.Width, lcl_desc.Height as u64),
            ));
        }
        if p_desc.Alignment == 0 {
            if p_desc.Layout == D3D12_TEXTURE_LAYOUT_64KB_UNDEFINED_SWIZZLE
                || p_desc.Layout == D3D12_TEXTURE_LAYOUT_64KB_STANDARD_SWIZZLE
            {
                lcl_desc.Alignment = D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64;
            } else {
                lcl_desc.Alignment = if p_desc.SampleDesc.Count > 1 {
                    D3D12_DEFAULT_MSAA_RESOURCE_PLACEMENT_ALIGNMENT as u64
                } else {
                    D3D12_DEFAULT_RESOURCE_PLACEMENT_ALIGNMENT as u64
                };
            }
        }
        lcl_desc
    } else {
        p_desc
    }
}

//------------------------------------------------------------------------------------------------
pub trait CD3DX12_VIEW_INSTANCING_DESC {
    fn new(
        view_instance_count: u32,
        view_instance_locations: *const D3D12_VIEW_INSTANCE_LOCATION,
        flags: D3D12_VIEW_INSTANCING_FLAGS,
    ) -> D3D12_VIEW_INSTANCING_DESC {
        D3D12_VIEW_INSTANCING_DESC {
            ViewInstanceCount: view_instance_count,
            pViewInstanceLocations: view_instance_locations,
            Flags: flags,
        }
    }
    fn default() -> D3D12_VIEW_INSTANCING_DESC {
        D3D12_VIEW_INSTANCING_DESC {
            ViewInstanceCount: 0,
            pViewInstanceLocations: std::ptr::null(),
            Flags: D3D12_VIEW_INSTANCING_FLAG_NONE,
        }
    }
}
impl CD3DX12_VIEW_INSTANCING_DESC for D3D12_VIEW_INSTANCING_DESC {}
//------------------------------------------------------------------------------------------------
pub trait CD3DX12_RT_FORMAT_ARRAY {
    fn new(formats: &[DXGI_FORMAT]) -> D3D12_RT_FORMAT_ARRAY {
        // FIXME: Use const generics if possible
        debug_assert!(formats.len() <= 8 /* D3D12_RT_FORMAT_ARRAY::RTFormats.len() */);
        let mut maybe: MaybeUninit<D3D12_RT_FORMAT_ARRAY> = MaybeUninit::uninit();
        // assumes ARRAY_SIZE(pFormats) == ARRAY_SIZE(RTFormats)
        unsafe {
            let ptr = maybe.as_mut_ptr();
            copy_nonoverlapping(
                formats.as_ptr(),
                addr_of_mut!((*ptr).RTFormats).cast(),
                formats.len(),
            );
            addr_of_mut!((*ptr).NumRenderTargets).write(formats.len() as u32);
        }
        unsafe { maybe.assume_init() }
    }
}
