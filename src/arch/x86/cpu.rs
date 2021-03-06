use void::Void;

use io::{self, Write};

pub use cpu::*;

use arch::keyboard::Keyboard;


static DEFAULT_KEYBOARD: Keyboard = Keyboard {
  callback:     ::put_char,
  control_port: Port(0x64),
  data_port:    Port(0x60),
};

pub unsafe fn init() {
  set_gdt(&*GDT);

  // Reload segment registers after lgdt
  set_cs(SegmentSelector::new(1, PrivilegeLevel::Ring0));

  let ds = SegmentSelector::new(2, PrivilegeLevel::Ring0);
  set_ds(ds);
  set_es(ds);
  set_fs(ds);
  set_gs(ds);
  set_ss(ds);

  PIC::master().remap_to(0x20);
  PIC::slave().remap_to(0x28);

  set_idt(&*IDT);
}

fn acknowledge_irq(_: u32) {
  PIC::master().control_port.out8(0x20); //TODO(ryan) ugly and only for master PIC
}

pub unsafe fn test_interrupt() {
  asm!("int 0x15" :::: "volatile", "intel");
}

macro_rules! make_handler {
  ($num:expr, $name:ident, $body:expr) => {{
    fn body () {
      $body
    }
    #[naked]
    unsafe extern "C" fn $name () {
      asm!(concat!(
        "push esp",                    "\n\t",
        "mov ebp, esp",                "\n\t",
        "pusha",                       "\n\t",

        "call  $0",                    "\n\t",

        "popa",                        "\n\t",
        "leave",                       "\n\t",
        "iretd",                       "\n\t")
           :: "s" (body as fn()) :: "volatile", "intel");
    }
    IdtEntry::new($name, PrivilegeLevel::Ring0, true)
  }};
  ($num:expr, $name:ident, EX, $title:expr) => {
    make_handler!($num, $name, {
      panic!("Exception {:#04x}: {}", $num, $title)
    })
  };
  ($num:expr, $name:ident) => {
    make_handler!($num, $name, {
      panic!("interrupt with no handler: {:#04x}", $num)
    })
  }
}

// TODO should be real statics
lazy_static! {

  static ref GDT: [GdtEntry; 3] = {[
    GdtEntry::NULL,
    GdtEntry::new(0 as *const (),
                  0xFFFFFFFF,
                  GdtAccess::Executable | GdtAccess::NotTss,
                  PrivilegeLevel::Ring0),
    GdtEntry::new(0 as *const (),
                  0xFFFFFFFF,
                  GdtAccess::Writable | GdtAccess::NotTss,
                  PrivilegeLevel::Ring0),
    //gdt.add_entry( = {.base=&myTss, .limit=sizeof(myTss), .type=0x89}; // You can use LTR(0x18)
  ]};

  static ref IDT: [IdtEntry; 256] = {[
    make_handler!(0x00, interrupt_handler_0x00, EX, "Divide by zero"),
    make_handler!(0x01, interrupt_handler_0x01, EX, "Debug"),
    make_handler!(0x02, interrupt_handler_0x02, EX, "Non-maskable Interrupt"),
    make_handler!(0x03, interrupt_handler_0x03, EX, "Breakpoint"),
    make_handler!(0x04, interrupt_handler_0x04, EX, "Overflow"),
    make_handler!(0x05, interrupt_handler_0x05, EX, "Bound Range Exceeded"),
    make_handler!(0x06, interrupt_handler_0x06, EX, "Invalid Opcode"),
    make_handler!(0x07, interrupt_handler_0x07, EX, "Device Not Available"),
    make_handler!(0x08, interrupt_handler_0x08, EX, "Double Fault"),
    make_handler!(0x09, interrupt_handler_0x09),
    make_handler!(0x0a, interrupt_handler_0x0a, EX, "Invalid TSS"),
    make_handler!(0x0b, interrupt_handler_0x0b, EX, "Segment Not Present"),
    make_handler!(0x0c, interrupt_handler_0x0c, EX, "Stack-Segment Fault"),
    make_handler!(0x0d, interrupt_handler_0x0d, EX, "General Protection Fault"),
    make_handler!(0x0e, interrupt_handler_0x0e, EX, "Page Fault"),
    make_handler!(0x0f, interrupt_handler_0x0f, EX, "x87 Floating Point Exception"),

    make_handler!(0x10, interrupt_handler_0x10),
    make_handler!(0x11, interrupt_handler_0x11),
    make_handler!(0x12, interrupt_handler_0x12),
    make_handler!(0x13, interrupt_handler_0x13),
    make_handler!(0x14, interrupt_handler_0x14),
    make_handler!(0x15, interrupt_handler_0x15, {
      debug!("In test interrupt handler");
    }),
    make_handler!(0x16, interrupt_handler_0x16),
    make_handler!(0x17, interrupt_handler_0x17),
    make_handler!(0x18, interrupt_handler_0x18),
    make_handler!(0x19, interrupt_handler_0x19),
    make_handler!(0x1a, interrupt_handler_0x1a),
    make_handler!(0x1b, interrupt_handler_0x1b),
    make_handler!(0x1c, interrupt_handler_0x1c),
    make_handler!(0x1d, interrupt_handler_0x1d),
    make_handler!(0x1e, interrupt_handler_0x1e),
    make_handler!(0x1f, interrupt_handler_0x1f),

    make_handler!(0x20, interrupt_handler_0x20, {
      // Timer, just ignore
    }),
    make_handler!(0x21, interrupt_handler_0x21, {
      DEFAULT_KEYBOARD.got_interrupted();
      acknowledge_irq(0x21);
    }),
    make_handler!(0x22, interrupt_handler_0x22),
    make_handler!(0x23, interrupt_handler_0x23),
    make_handler!(0x24, interrupt_handler_0x24),
    make_handler!(0x25, interrupt_handler_0x25),
    make_handler!(0x26, interrupt_handler_0x26),
    make_handler!(0x27, interrupt_handler_0x27),
    make_handler!(0x28, interrupt_handler_0x28),
    make_handler!(0x29, interrupt_handler_0x29),
    make_handler!(0x2a, interrupt_handler_0x2a),
    make_handler!(0x2b, interrupt_handler_0x2b),
    make_handler!(0x2c, interrupt_handler_0x2c),
    make_handler!(0x2d, interrupt_handler_0x2d),
    make_handler!(0x2e, interrupt_handler_0x2e),
    make_handler!(0x2f, interrupt_handler_0x2f),

    make_handler!(0x30, interrupt_handler_0x30),
    make_handler!(0x31, interrupt_handler_0x31),
    make_handler!(0x32, interrupt_handler_0x32),
    make_handler!(0x33, interrupt_handler_0x33),
    make_handler!(0x34, interrupt_handler_0x34),
    make_handler!(0x35, interrupt_handler_0x35),
    make_handler!(0x36, interrupt_handler_0x36),
    make_handler!(0x37, interrupt_handler_0x37),
    make_handler!(0x38, interrupt_handler_0x38),
    make_handler!(0x39, interrupt_handler_0x39),
    make_handler!(0x3a, interrupt_handler_0x3a),
    make_handler!(0x3b, interrupt_handler_0x3b),
    make_handler!(0x3c, interrupt_handler_0x3c),
    make_handler!(0x3d, interrupt_handler_0x3d),
    make_handler!(0x3e, interrupt_handler_0x3e),
    make_handler!(0x3f, interrupt_handler_0x3f),

    make_handler!(0x40, interrupt_handler_0x40),
    make_handler!(0x41, interrupt_handler_0x41),
    make_handler!(0x42, interrupt_handler_0x42),
    make_handler!(0x43, interrupt_handler_0x43),
    make_handler!(0x44, interrupt_handler_0x44),
    make_handler!(0x45, interrupt_handler_0x45),
    make_handler!(0x46, interrupt_handler_0x46),
    make_handler!(0x47, interrupt_handler_0x47),
    make_handler!(0x48, interrupt_handler_0x48),
    make_handler!(0x49, interrupt_handler_0x49),
    make_handler!(0x4a, interrupt_handler_0x4a),
    make_handler!(0x4b, interrupt_handler_0x4b),
    make_handler!(0x4c, interrupt_handler_0x4c),
    make_handler!(0x4d, interrupt_handler_0x4d),
    make_handler!(0x4e, interrupt_handler_0x4e),
    make_handler!(0x4f, interrupt_handler_0x4f),

    make_handler!(0x50, interrupt_handler_0x50),
    make_handler!(0x51, interrupt_handler_0x51),
    make_handler!(0x52, interrupt_handler_0x52),
    make_handler!(0x53, interrupt_handler_0x53),
    make_handler!(0x54, interrupt_handler_0x54),
    make_handler!(0x55, interrupt_handler_0x55),
    make_handler!(0x56, interrupt_handler_0x56),
    make_handler!(0x57, interrupt_handler_0x57),
    make_handler!(0x58, interrupt_handler_0x58),
    make_handler!(0x59, interrupt_handler_0x59),
    make_handler!(0x5a, interrupt_handler_0x5a),
    make_handler!(0x5b, interrupt_handler_0x5b),
    make_handler!(0x5c, interrupt_handler_0x5c),
    make_handler!(0x5d, interrupt_handler_0x5d),
    make_handler!(0x5e, interrupt_handler_0x5e),
    make_handler!(0x5f, interrupt_handler_0x5f),

    make_handler!(0x60, interrupt_handler_0x60),
    make_handler!(0x61, interrupt_handler_0x61),
    make_handler!(0x62, interrupt_handler_0x62),
    make_handler!(0x63, interrupt_handler_0x63),
    make_handler!(0x64, interrupt_handler_0x64),
    make_handler!(0x65, interrupt_handler_0x65),
    make_handler!(0x66, interrupt_handler_0x66),
    make_handler!(0x67, interrupt_handler_0x67),
    make_handler!(0x68, interrupt_handler_0x68),
    make_handler!(0x69, interrupt_handler_0x69),
    make_handler!(0x6a, interrupt_handler_0x6a),
    make_handler!(0x6b, interrupt_handler_0x6b),
    make_handler!(0x6c, interrupt_handler_0x6c),
    make_handler!(0x6d, interrupt_handler_0x6d),
    make_handler!(0x6e, interrupt_handler_0x6e),
    make_handler!(0x6f, interrupt_handler_0x6f),

    make_handler!(0x70, interrupt_handler_0x70),
    make_handler!(0x71, interrupt_handler_0x71),
    make_handler!(0x72, interrupt_handler_0x72),
    make_handler!(0x73, interrupt_handler_0x73),
    make_handler!(0x74, interrupt_handler_0x74),
    make_handler!(0x75, interrupt_handler_0x75),
    make_handler!(0x76, interrupt_handler_0x76),
    make_handler!(0x77, interrupt_handler_0x77),
    make_handler!(0x78, interrupt_handler_0x78),
    make_handler!(0x79, interrupt_handler_0x79),
    make_handler!(0x7a, interrupt_handler_0x7a),
    make_handler!(0x7b, interrupt_handler_0x7b),
    make_handler!(0x7c, interrupt_handler_0x7c),
    make_handler!(0x7d, interrupt_handler_0x7d),
    make_handler!(0x7e, interrupt_handler_0x7e),
    make_handler!(0x7f, interrupt_handler_0x7f),

    make_handler!(0x80, interrupt_handler_0x80),
    make_handler!(0x81, interrupt_handler_0x81),
    make_handler!(0x82, interrupt_handler_0x82),
    make_handler!(0x83, interrupt_handler_0x83),
    make_handler!(0x84, interrupt_handler_0x84),
    make_handler!(0x85, interrupt_handler_0x85),
    make_handler!(0x86, interrupt_handler_0x86),
    make_handler!(0x87, interrupt_handler_0x87),
    make_handler!(0x88, interrupt_handler_0x88),
    make_handler!(0x89, interrupt_handler_0x89),
    make_handler!(0x8a, interrupt_handler_0x8a),
    make_handler!(0x8b, interrupt_handler_0x8b),
    make_handler!(0x8c, interrupt_handler_0x8c),
    make_handler!(0x8d, interrupt_handler_0x8d),
    make_handler!(0x8e, interrupt_handler_0x8e),
    make_handler!(0x8f, interrupt_handler_0x8f),

    make_handler!(0x90, interrupt_handler_0x90),
    make_handler!(0x91, interrupt_handler_0x91),
    make_handler!(0x92, interrupt_handler_0x92),
    make_handler!(0x93, interrupt_handler_0x93),
    make_handler!(0x94, interrupt_handler_0x94),
    make_handler!(0x95, interrupt_handler_0x95),
    make_handler!(0x96, interrupt_handler_0x96),
    make_handler!(0x97, interrupt_handler_0x97),
    make_handler!(0x98, interrupt_handler_0x98),
    make_handler!(0x99, interrupt_handler_0x99),
    make_handler!(0x9a, interrupt_handler_0x9a),
    make_handler!(0x9b, interrupt_handler_0x9b),
    make_handler!(0x9c, interrupt_handler_0x9c),
    make_handler!(0x9d, interrupt_handler_0x9d),
    make_handler!(0x9e, interrupt_handler_0x9e),
    make_handler!(0x9f, interrupt_handler_0x9f),

    make_handler!(0xa0, interrupt_handler_0xa0),
    make_handler!(0xa1, interrupt_handler_0xa1),
    make_handler!(0xa2, interrupt_handler_0xa2),
    make_handler!(0xa3, interrupt_handler_0xa3),
    make_handler!(0xa4, interrupt_handler_0xa4),
    make_handler!(0xa5, interrupt_handler_0xa5),
    make_handler!(0xa6, interrupt_handler_0xa6),
    make_handler!(0xa7, interrupt_handler_0xa7),
    make_handler!(0xa8, interrupt_handler_0xa8),
    make_handler!(0xa9, interrupt_handler_0xa9),
    make_handler!(0xaa, interrupt_handler_0xaa),
    make_handler!(0xab, interrupt_handler_0xab),
    make_handler!(0xac, interrupt_handler_0xac),
    make_handler!(0xad, interrupt_handler_0xad),
    make_handler!(0xae, interrupt_handler_0xae),
    make_handler!(0xaf, interrupt_handler_0xaf),

    make_handler!(0xb0, interrupt_handler_0xb0),
    make_handler!(0xb1, interrupt_handler_0xb1),
    make_handler!(0xb2, interrupt_handler_0xb2),
    make_handler!(0xb3, interrupt_handler_0xb3),
    make_handler!(0xb4, interrupt_handler_0xb4),
    make_handler!(0xb5, interrupt_handler_0xb5),
    make_handler!(0xb6, interrupt_handler_0xb6),
    make_handler!(0xb7, interrupt_handler_0xb7),
    make_handler!(0xb8, interrupt_handler_0xb8),
    make_handler!(0xb9, interrupt_handler_0xb9),
    make_handler!(0xba, interrupt_handler_0xba),
    make_handler!(0xbb, interrupt_handler_0xbb),
    make_handler!(0xbc, interrupt_handler_0xbc),
    make_handler!(0xbd, interrupt_handler_0xbd),
    make_handler!(0xbe, interrupt_handler_0xbe),
    make_handler!(0xbf, interrupt_handler_0xbf),

    make_handler!(0xc0, interrupt_handler_0xc0),
    make_handler!(0xc1, interrupt_handler_0xc1),
    make_handler!(0xc2, interrupt_handler_0xc2),
    make_handler!(0xc3, interrupt_handler_0xc3),
    make_handler!(0xc4, interrupt_handler_0xc4),
    make_handler!(0xc5, interrupt_handler_0xc5),
    make_handler!(0xc6, interrupt_handler_0xc6),
    make_handler!(0xc7, interrupt_handler_0xc7),
    make_handler!(0xc8, interrupt_handler_0xc8),
    make_handler!(0xc9, interrupt_handler_0xc9),
    make_handler!(0xca, interrupt_handler_0xca),
    make_handler!(0xcb, interrupt_handler_0xcb),
    make_handler!(0xcc, interrupt_handler_0xcc),
    make_handler!(0xcd, interrupt_handler_0xcd),
    make_handler!(0xce, interrupt_handler_0xce),
    make_handler!(0xcf, interrupt_handler_0xcf),

    make_handler!(0xd0, interrupt_handler_0xd0),
    make_handler!(0xd1, interrupt_handler_0xd1),
    make_handler!(0xd2, interrupt_handler_0xd2),
    make_handler!(0xd3, interrupt_handler_0xd3),
    make_handler!(0xd4, interrupt_handler_0xd4),
    make_handler!(0xd5, interrupt_handler_0xd5),
    make_handler!(0xd6, interrupt_handler_0xd6),
    make_handler!(0xd7, interrupt_handler_0xd7),
    make_handler!(0xd8, interrupt_handler_0xd8),
    make_handler!(0xd9, interrupt_handler_0xd9),
    make_handler!(0xda, interrupt_handler_0xda),
    make_handler!(0xdb, interrupt_handler_0xdb),
    make_handler!(0xdc, interrupt_handler_0xdc),
    make_handler!(0xdd, interrupt_handler_0xdd),
    make_handler!(0xde, interrupt_handler_0xde),
    make_handler!(0xdf, interrupt_handler_0xdf),

    make_handler!(0xe0, interrupt_handler_0xe0),
    make_handler!(0xe1, interrupt_handler_0xe1),
    make_handler!(0xe2, interrupt_handler_0xe2),
    make_handler!(0xe3, interrupt_handler_0xe3),
    make_handler!(0xe4, interrupt_handler_0xe4),
    make_handler!(0xe5, interrupt_handler_0xe5),
    make_handler!(0xe6, interrupt_handler_0xe6),
    make_handler!(0xe7, interrupt_handler_0xe7),
    make_handler!(0xe8, interrupt_handler_0xe8),
    make_handler!(0xe9, interrupt_handler_0xe9),
    make_handler!(0xea, interrupt_handler_0xea),
    make_handler!(0xeb, interrupt_handler_0xeb),
    make_handler!(0xec, interrupt_handler_0xec),
    make_handler!(0xed, interrupt_handler_0xed),
    make_handler!(0xee, interrupt_handler_0xee),
    make_handler!(0xef, interrupt_handler_0xef),

    make_handler!(0xf0, interrupt_handler_0xf0),
    make_handler!(0xf1, interrupt_handler_0xf1),
    make_handler!(0xf2, interrupt_handler_0xf2),
    make_handler!(0xf3, interrupt_handler_0xf3),
    make_handler!(0xf4, interrupt_handler_0xf4),
    make_handler!(0xf5, interrupt_handler_0xf5),
    make_handler!(0xf6, interrupt_handler_0xf6),
    make_handler!(0xf7, interrupt_handler_0xf7),
    make_handler!(0xf8, interrupt_handler_0xf8),
    make_handler!(0xf9, interrupt_handler_0xf9),
    make_handler!(0xfa, interrupt_handler_0xfa),
    make_handler!(0xfb, interrupt_handler_0xfb),
    make_handler!(0xfc, interrupt_handler_0xfc),
    make_handler!(0xfd, interrupt_handler_0xfd),
    make_handler!(0xfe, interrupt_handler_0xfe),
    make_handler!(0xff, interrupt_handler_0xff),
  ]};

}



struct PIC {
  control_port: Port,
  mask_port: Port,
  is_master: bool
}

impl PIC {

  fn master() -> PIC {
    PIC { control_port: Port::new(0x20), mask_port: Port::new(0x21), is_master: true}
  }

  fn slave() -> PIC {
    PIC { control_port: Port::new(0xA0), mask_port: Port::new(0xA1), is_master: false}
  }

  unsafe fn remap_to(&mut self, start: u8) {
    let icw1 = 0x11;
    let icw4 = 0x1;
    let enable_all = 0x00;
    let typ = if self.is_master { 0x2 } else { 0x4 };

    self.control_port.out8(icw1);
    self.mask_port.write(&[start, typ, icw4, enable_all]).ok();
  }

}



#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Debug)]
pub struct Port(u16);

impl Port {

  pub const fn new(number: u16) -> Port {
    Port(number)
  }

  pub fn in8(self) -> u8 {
    unsafe { ::cpu::in8(self.0) }
  }

  pub fn out8(self, num: u8) {
    unsafe { ::cpu::out8(self.0, num) }
  }

  pub fn in16(self) -> u16 {
    unsafe { ::cpu::in16(self.0) }
  }

  pub fn out16(self, num: u16) {
    unsafe { ::cpu::out16(self.0, num) }
  }

  pub fn in32(self) -> u32 {
    unsafe { ::cpu::in32(self.0) }
  }

  pub fn out32(self, num: u32) {
    unsafe { ::cpu::out32(self.0, num) }
  }

  pub fn io_wait() {
    Port::new(0x80).out8(0);
  }

}

impl io::Read for Port
{
  type Err = Void;

  fn read(&mut self, buf: &mut [u8]) -> Result<usize, Void> {
    Ok(match *buf {
      []               => 0,
      [ref mut a, _..] => {
        *a = self.in8();
        1
      }
    })
  }

  fn read_all<E>(&mut self, buf: &mut [u8]) -> Result<(), E> {
    for el in buf.iter_mut() {
      *el = self.in8();
    }
    Ok(())
  }
}

impl io::Write for Port
{
  type Err = Void;

  fn write(&mut self, buf: &[u8]) -> Result<usize, Void> {
    Ok(match *buf {
      []       => 0,
      [a, _..] => {
        self.out8(a);
        1
      }
    })
  }

  fn write_all<E>(&mut self, buf: &[u8]) -> Result<(), E> {
    for el in buf.iter() {
      self.out8(*el);
    }
    Ok(())
  }
}
