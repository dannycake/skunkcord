// Copyright (c) Skunk Ventures LLC
// Last modified: 2025-03-07
// SPDX-License-Identifier: MIT

//! Browser data constants and helpers for fingerprint emulation

use serde::{Deserialize, Serialize};

/// Common browser configurations used for fingerprint generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub name: String,
    pub vendor: String,
    pub platform: String,
    pub cookie_enabled: bool,
    pub do_not_track: Option<String>,
    pub plugins: Vec<PluginInfo>,
    pub mime_types: Vec<MimeTypeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub filename: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MimeTypeInfo {
    pub type_: String,
    pub suffixes: String,
    pub description: String,
}

impl BrowserConfig {
    /// Get Chrome browser configuration
    pub fn chrome() -> Self {
        Self {
            name: "Netscape".to_string(),
            vendor: "Google Inc.".to_string(),
            platform: "Win32".to_string(),
            cookie_enabled: true,
            do_not_track: None,
            plugins: vec![
                PluginInfo {
                    name: "PDF Viewer".to_string(),
                    filename: "internal-pdf-viewer".to_string(),
                    description: "Portable Document Format".to_string(),
                },
                PluginInfo {
                    name: "Chrome PDF Viewer".to_string(),
                    filename: "internal-pdf-viewer".to_string(),
                    description: "Portable Document Format".to_string(),
                },
                PluginInfo {
                    name: "Chromium PDF Viewer".to_string(),
                    filename: "internal-pdf-viewer".to_string(),
                    description: "Portable Document Format".to_string(),
                },
                PluginInfo {
                    name: "Microsoft Edge PDF Viewer".to_string(),
                    filename: "internal-pdf-viewer".to_string(),
                    description: "Portable Document Format".to_string(),
                },
                PluginInfo {
                    name: "WebKit built-in PDF".to_string(),
                    filename: "internal-pdf-viewer".to_string(),
                    description: "Portable Document Format".to_string(),
                },
            ],
            mime_types: vec![
                MimeTypeInfo {
                    type_: "application/pdf".to_string(),
                    suffixes: "pdf".to_string(),
                    description: "Portable Document Format".to_string(),
                },
                MimeTypeInfo {
                    type_: "text/pdf".to_string(),
                    suffixes: "pdf".to_string(),
                    description: "Portable Document Format".to_string(),
                },
            ],
        }
    }

    /// Get Firefox browser configuration
    pub fn firefox() -> Self {
        Self {
            name: "Netscape".to_string(),
            vendor: "".to_string(),
            platform: "Win32".to_string(),
            cookie_enabled: true,
            do_not_track: Some("unspecified".to_string()),
            plugins: vec![],
            mime_types: vec![],
        }
    }
}

/// WebGL parameters for fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGLParams {
    pub vendor: String,
    pub renderer: String,
    pub version: String,
    pub shading_language_version: String,
    pub extensions: Vec<String>,
    pub max_texture_size: u32,
    pub max_viewport_dims: (u32, u32),
    pub max_vertex_attribs: u32,
    pub max_vertex_uniform_vectors: u32,
    pub max_varying_vectors: u32,
    pub max_fragment_uniform_vectors: u32,
    pub aliased_line_width_range: (f32, f32),
    pub aliased_point_size_range: (f32, f32),
}

impl Default for WebGLParams {
    fn default() -> Self {
        Self {
            vendor: "Google Inc. (NVIDIA)".to_string(),
            renderer: "ANGLE (NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0)".to_string(),
            version: "WebGL 2.0 (OpenGL ES 3.0 Chromium)".to_string(),
            shading_language_version: "WebGL GLSL ES 3.00 (OpenGL ES GLSL ES 3.0 Chromium)"
                .to_string(),
            extensions: vec![
                "ANGLE_instanced_arrays".to_string(),
                "EXT_blend_minmax".to_string(),
                "EXT_color_buffer_half_float".to_string(),
                "EXT_disjoint_timer_query".to_string(),
                "EXT_float_blend".to_string(),
                "EXT_frag_depth".to_string(),
                "EXT_shader_texture_lod".to_string(),
                "EXT_texture_compression_bptc".to_string(),
                "EXT_texture_compression_rgtc".to_string(),
                "EXT_texture_filter_anisotropic".to_string(),
                "EXT_sRGB".to_string(),
                "OES_element_index_uint".to_string(),
                "OES_fbo_render_mipmap".to_string(),
                "OES_standard_derivatives".to_string(),
                "OES_texture_float".to_string(),
                "OES_texture_float_linear".to_string(),
                "OES_texture_half_float".to_string(),
                "OES_texture_half_float_linear".to_string(),
                "OES_vertex_array_object".to_string(),
                "WEBGL_color_buffer_float".to_string(),
                "WEBGL_compressed_texture_s3tc".to_string(),
                "WEBGL_compressed_texture_s3tc_srgb".to_string(),
                "WEBGL_debug_renderer_info".to_string(),
                "WEBGL_debug_shaders".to_string(),
                "WEBGL_depth_texture".to_string(),
                "WEBGL_draw_buffers".to_string(),
                "WEBGL_lose_context".to_string(),
                "WEBGL_multi_draw".to_string(),
            ],
            max_texture_size: 16384,
            max_viewport_dims: (32767, 32767),
            max_vertex_attribs: 16,
            max_vertex_uniform_vectors: 4096,
            max_varying_vectors: 30,
            max_fragment_uniform_vectors: 1024,
            aliased_line_width_range: (1.0, 1.0),
            aliased_point_size_range: (1.0, 1024.0),
        }
    }
}

/// Audio context fingerprint parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFingerprint {
    pub sample_rate: u32,
    pub max_channel_count: u32,
    pub channel_count: u32,
    pub channel_count_mode: String,
    pub channel_interpretation: String,
    pub state: String,
    pub oscillator_value: f64,
}

impl Default for AudioFingerprint {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            max_channel_count: 2,
            channel_count: 2,
            channel_count_mode: "max".to_string(),
            channel_interpretation: "speakers".to_string(),
            state: "running".to_string(),
            oscillator_value: 0.0,
        }
    }
}

/// Screen and display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub avail_width: u32,
    pub avail_height: u32,
    pub color_depth: u8,
    pub pixel_depth: u8,
    pub pixel_ratio: f64,
    pub orientation_type: String,
    pub orientation_angle: u16,
}

impl Default for ScreenInfo {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            avail_width: 1920,
            avail_height: 1040,
            color_depth: 24,
            pixel_depth: 24,
            pixel_ratio: 1.0,
            orientation_type: "landscape-primary".to_string(),
            orientation_angle: 0,
        }
    }
}

/// Client hints data for modern browsers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHints {
    pub brands: Vec<BrandVersion>,
    pub mobile: bool,
    pub platform: String,
    pub platform_version: String,
    pub architecture: String,
    pub bitness: String,
    pub model: String,
    pub full_version_list: Vec<BrandVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandVersion {
    pub brand: String,
    pub version: String,
}

impl Default for ClientHints {
    fn default() -> Self {
        Self {
            brands: vec![
                BrandVersion {
                    brand: "Chromium".to_string(),
                    version: "120".to_string(),
                },
                BrandVersion {
                    brand: "Google Chrome".to_string(),
                    version: "120".to_string(),
                },
                BrandVersion {
                    brand: "Not-A.Brand".to_string(),
                    version: "24".to_string(),
                },
            ],
            mobile: false,
            platform: "Windows".to_string(),
            platform_version: "15.0.0".to_string(),
            architecture: "x86".to_string(),
            bitness: "64".to_string(),
            model: "".to_string(),
            full_version_list: vec![
                BrandVersion {
                    brand: "Chromium".to_string(),
                    version: "120.0.6099.130".to_string(),
                },
                BrandVersion {
                    brand: "Google Chrome".to_string(),
                    version: "120.0.6099.130".to_string(),
                },
            ],
        }
    }
}
