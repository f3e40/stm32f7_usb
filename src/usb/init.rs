use super::*;
use stm32f7::{embedded};
use embedded::interfaces::gpio::Gpio;
use board::rcc::Rcc;
use board::nvic::Nvic;
use board::otg_hs_device::OtgHsDevice;
use board::otg_hs_global::OtgHsGlobal;

pub fn init(rcc: &mut Rcc, gpio: &mut Gpio, otg_hs_global: &'static mut OtgHsGlobal, otg_hs_device: &'static mut OtgHsDevice, nvic: &'static mut Nvic) -> Usb {
	rcc.ahb1enr.update(|r| r.set_otghsen(true));
	rcc.ahb1enr.update(|r| r.set_otghsulpien(true));
	
	init_pins(gpio);

	//core init
	otg_hs_global.otg_hs_gccfg.update(|r| r.set_pwrdwn(false));

	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_physel(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_tsdps(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_ulpifsls(false));

	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_ulpievbusd(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_ulpievbusi(false));
	
	while ! otg_hs_global.otg_hs_grstctl.read().ahbidl() {};
	otg_hs_global.otg_hs_grstctl.update(|r| r.set_csrst(true));
	while otg_hs_global.otg_hs_grstctl.read().csrst() {};

	//options

	otg_hs_global.otg_hs_gahbcfg.update(|r| r.set_gint(true));
	otg_hs_global.otg_hs_gahbcfg.update(|r| r.set_ptxfelvl(true)); //completely empty

	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_hnpcap(false));
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_srpcap(false));
	//otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_tocal()); //not necessary for hs?
	otg_hs_global.otg_hs_gusbcfg.update(|r| r.set_trdt(0x9)); //only valid value for hs?

	// Wait till we enter device mode
	//TODO
	//while otg_hs_global.otg_hs_gintsts.read().cmod() { /*sleep*/ }

	otg_hs_global.otg_hs_gccfg.update(|r| r.set_vbden(true));
	//otg_hs_global.otg_hs_pcgcctl.write(embedded_stm32f7::otg_hs_global::OtgHsPcgcctl::default());
	//device mode init
	//options
	otg_hs_device.otg_hs_dcfg.update(|r| r.set_dspd(0)); // high speed
	otg_hs_device.otg_hs_dcfg.update(|r| r.set_nzlsohsk(false)); //no clue

	otg_hs_device.otg_hs_dctl.update(|r| r.set_sdis(false));

	unsafe { interrupt::init(otg_hs_global,	otg_hs_device, nvic); }
	Usb {
	}
}

fn init_pins(gpio: &mut Gpio) {
	use embedded::interfaces::gpio::Port::*;
	use embedded::interfaces::gpio::Pin::*;
	use embedded::interfaces::gpio::{OutputType, OutputSpeed, AlternateFunction, Resistor};
	let pins = [
		(PortA, Pin3), 	// D0
		(PortB, Pin0),	// D1
		(PortB, Pin1),	// D2
		(PortB, Pin10),	// D3
		(PortB, Pin11),	// D4
		(PortB, Pin12),	// D5
		(PortB, Pin13),	// D6
		(PortB, Pin5),	// D7
		(PortA, Pin5), 	// CLK
		(PortC, Pin0),	// STP
		(PortC, Pin2),	// DIR
		(PortH, Pin4),	// NXT
	];
	match gpio.to_alternate_function_all(&pins,
			AlternateFunction::AF10,
			OutputType::PushPull,
			OutputSpeed::High,
			Resistor::NoPull) {
		Ok(_) => (),
		Err(embedded::interfaces::gpio::Error::PinAlreadyInUse(_)) => {
			unsafe { asm!("bkpt 0xAB"); }
		},
	}
}

