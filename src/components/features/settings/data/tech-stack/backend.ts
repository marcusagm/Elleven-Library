import { TechCategory } from './types';

export const backendEngine: TechCategory = {
    title: 'Backend Native Engine (Rust)',
    items: [
        {
            name: 'asefile',
            description: 'Aseprite format parsing (.ase, .aseprite).',
            url: 'https://crates.io/crates/asefile'
        },
        {
            name: 'axum & tower-http',
            description: 'Web application framework & middleware.',
            url: 'https://tokio.rs/#axum'
        },
        {
            name: 'base64',
            description: 'Base64 encoding/decoding.',
            url: 'https://crates.io/crates/base64'
        },
        {
            name: 'byteorder',
            description: 'Reading/writing numbers in big/little endian.',
            url: 'https://crates.io/crates/byteorder'
        },
        {
            name: 'chrono',
            description: 'Date and time library for Rust.',
            url: 'https://crates.io/crates/chrono'
        },
        {
            name: 'fast_image_resize',
            description: 'SIMD-accelerated image resize.',
            url: 'https://crates.io/crates/fast_image_resize'
        },
        {
            name: 'flate2 & zstd',
            description: 'Compression algorithms.',
            url: 'https://crates.io/crates/zstd'
        },
        {
            name: 'image & imagesize',
            description: 'Image processing and dimension detection.',
            url: 'https://crates.io/crates/image'
        },
        {
            name: 'infer & mime_guess',
            description: 'MIME types discovery and magic bytes.',
            url: 'https://crates.io/crates/infer'
        },
        {
            name: 'kmeans_colors & palette',
            description: 'Color clustering and space conversion.',
            url: 'https://crates.io/crates/kmeans_colors'
        },
        {
            name: 'memmap2',
            description: 'Memory-mapped file API.',
            url: 'https://crates.io/crates/memmap2'
        },
        {
            name: 'notify',
            description: 'Cross-platform filesystem notification system.',
            url: 'https://crates.io/crates/notify'
        },
        {
            name: 'pdfium-render',
            description: 'High-level bindings to Google Pdfium.',
            url: 'https://crates.io/crates/pdfium-render'
        },
        {
            name: 'percent-encoding & urlencoding',
            description: 'URL encoding for strings.',
            url: 'https://crates.io/crates/urlencoding'
        },
        {
            name: 'psd',
            description: 'Parsing Photoshop PSD files.',
            url: 'https://crates.io/crates/psd'
        },
        {
            name: 'quick-xml',
            description: 'High performance XML reader/writer.',
            url: 'https://crates.io/crates/quick-xml'
        },
        {
            name: 'quickraw & rsraw',
            description: 'Camera RAW files processing/decoding.',
            url: 'https://crates.io/crates/quickraw'
        },
        {
            name: 'rayon',
            description: 'Data-parallelism library for Rust.',
            url: 'https://crates.io/crates/rayon'
        },
        {
            name: 'resvg & tiny-skia',
            description: 'SVG rendering engine.',
            url: 'https://crates.io/crates/resvg'
        },
        {
            name: 'rexif',
            description: 'EXIF metadata parsing.',
            url: 'https://crates.io/crates/rexif'
        },
        {
            name: 'serde & serde_json',
            description: 'Data serialization framework.',
            url: 'https://serde.rs/'
        },
        {
            name: 'sqlx (sqlite)',
            description: 'Asynchronous SQL toolkit.',
            url: 'https://crates.io/crates/sqlx'
        },
        {
            name: 'strum & strum_macros',
            description: 'Macros for working with enums.',
            url: 'https://crates.io/crates/strum'
        },
        {
            name: 'tauri-plugin-*',
            description: 'Plugins for CLI, dialogs, fs, HTTP, MCP, Opener, SQL.',
            url: 'https://github.com/tauri-apps/plugins-workspace'
        },
        {
            name: 'thiserror',
            description: 'Derive macro for standard Error trait.',
            url: 'https://crates.io/crates/thiserror'
        },
        {
            name: 'tokio & tokio-util',
            description: 'Asynchronous runtime and utilities.',
            url: 'https://tokio.rs/'
        },
        {
            name: 'tracing & tracing-subscriber',
            description: 'Application-level logging and profiling.',
            url: 'https://crates.io/crates/tracing'
        },
        {
            name: 'uuid',
            description: 'Unique identifiers generation.',
            url: 'https://crates.io/crates/uuid'
        },
        {
            name: 'wait-timeout',
            description: 'Process waiting with timeout.',
            url: 'https://crates.io/crates/wait-timeout'
        },
        {
            name: 'walkdir',
            description: 'Directory tree traversal recursively.',
            url: 'https://crates.io/crates/walkdir'
        },
        {
            name: 'webp & zune-jpeg',
            description: 'Specific fast image codecs (WebP, JPEG).',
            url: 'https://crates.io/crates/zune-jpeg'
        },
        {
            name: 'wuff',
            description: 'Wuffs font/image decoding (WOFF).',
            url: 'https://crates.io/crates/wuff'
        },
        {
            name: 'zip',
            description: 'ZIP archive reading/writing.',
            url: 'https://crates.io/crates/zip'
        }
    ]
};
