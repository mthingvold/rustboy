//! `LcdItem`: a QML item that paints the latest emulator frame with nearest-neighbour scaling.
//! QML sizes it to an integer multiple of 160×144.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtQuick/QQuickPaintedItem>);
        type QQuickPaintedItem;

        include!("cxx-qt-lib/qpainter.h");
        type QPainter = cxx_qt_lib::QPainter;
        include!("cxx-qt-lib/qrect.h");
        type QRect = cxx_qt_lib::QRect;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QQuickPaintedItem]
        #[qproperty(i32, lcd_palette)]
        type LcdItem = super::LcdItemRust;

        #[cxx_override]
        unsafe fn paint(self: Pin<&mut LcdItem>, painter: *mut QPainter);

        /// Schedule a repaint with the latest frame.
        #[qinvokable]
        fn refresh(self: Pin<&mut LcdItem>);
    }

    unsafe extern "RustQt" {
        #[inherit]
        fn update(self: Pin<&mut LcdItem>, rect: &QRect);
        #[inherit]
        fn width(self: &LcdItem) -> f64;
        #[inherit]
        fn height(self: &LcdItem) -> f64;
    }

    impl cxx_qt::Constructor<()> for LcdItem {}
}

use core::pin::Pin;

use cxx_qt_lib::{QImage, QImageFormat, QRect};

use crate::core::{LCD_HEIGHT, LCD_WIDTH};
use crate::emu_thread::FRAME;

/// LCD palettes as RGB, lightest to darkest: 0 = Green (default), 1 = Gray.
const PALETTES: [[[u8; 3]; 4]; 2] = [
    [
        [0x9B, 0xBC, 0x0F],
        [0x8B, 0xAC, 0x0F],
        [0x30, 0x62, 0x30],
        [0x0F, 0x38, 0x0F],
    ],
    [
        [0xC6, 0xC6, 0xB8],
        [0x9A, 0x9A, 0x8E],
        [0x56, 0x56, 0x4F],
        [0x1F, 0x1F, 0x1C],
    ],
];

/// Rust state behind `LcdItem`.
#[derive(Default)]
pub struct LcdItemRust {
    lcd_palette: i32,
}

impl cxx_qt::Constructor<()> for qobject::LcdItem {
    type NewArguments = ();
    // QQuickPaintedItem's constructor takes a QQuickItem parent, not a QObject; the QML
    // engine reparents the item after construction
    type BaseArguments = ();
    type InitializeArguments = ();

    fn route_arguments(_: ()) -> ((), (), ()) {
        ((), (), ())
    }

    fn new(_: ()) -> LcdItemRust {
        LcdItemRust::default()
    }
}

impl qobject::LcdItem {
    fn paint(self: Pin<&mut Self>, painter: *mut qobject::QPainter) {
        // Safety: Qt passes a valid, active painter for the duration of paint()
        let painter = match unsafe { painter.as_mut() } {
            Some(p) => unsafe { Pin::new_unchecked(p) },
            None => return,
        };

        let palette = &PALETTES[(self.lcd_palette.clamp(0, 1)) as usize];
        let mut rgbx = Vec::with_capacity(LCD_WIDTH * LCD_HEIGHT * 4);
        {
            let frame = FRAME.lock().unwrap_or_else(|e| e.into_inner());
            for shade in frame.iter() {
                let [r, g, b] = palette[(*shade & 3) as usize];
                rgbx.extend_from_slice(&[r, g, b, 0xFF]);
            }
        }

        // Safety: the buffer holds exactly width × height RGBX pixels
        let image = unsafe {
            QImage::from_raw_bytes(
                rgbx,
                LCD_WIDTH as i32,
                LCD_HEIGHT as i32,
                QImageFormat::Format_RGBX8888,
            )
        };
        let target = QRect::new(
            0,
            0,
            self.width().round() as i32,
            self.height().round() as i32,
        );
        // SmoothPixmapTransform is off by default, so scaling is nearest-neighbour
        painter.draw_image(&target, &image);
    }

    fn refresh(self: Pin<&mut Self>) {
        self.update(&QRect::default());
    }
}
