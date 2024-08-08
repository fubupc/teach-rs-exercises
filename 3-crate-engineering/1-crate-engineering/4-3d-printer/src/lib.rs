use std::marker::PhantomData;

use rand::Rng;

pub struct Printer3D<S> {
    _marker: PhantomData<S>,
}

/* States */

/// The 3D printer encountered an error and needs resetting
pub enum ErrorState {}
/// The 3D printer is waiting for a job
pub enum IdleState {}
/// The 3D printer is currently printing
pub enum PrintingState {}
/// The 3D printed product is ready
pub enum ProductReadyState {}

/// Check if we're out of filament
fn out_of_filament() -> bool {
    let rand: usize = rand::thread_rng().gen_range(0..100);
    rand > 95
}

impl<S> Printer3D<S> {
    pub fn into_state<T>(self) -> Printer3D<T> {
        Printer3D {
            _marker: PhantomData,
        }
    }
}

impl Printer3D<IdleState> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn start(self) -> Printer3D<PrintingState> {
        println!("Idle: start printing.");
        self.into_state()
    }
}

impl Printer3D<PrintingState> {
    pub fn print(self) -> Result<Printer3D<ProductReadyState>, Printer3D<ErrorState>> {
        if out_of_filament() {
            println!("Printing: out of filament.");
            Err(self.into_state())
        } else {
            println!("Printing: product ready.");
            Ok(self.into_state())
        }
    }
}

impl Printer3D<ProductReadyState> {
    pub fn retrieve(self) -> Printer3D<IdleState> {
        println!("Product Ready: product retrieved.");
        self.into_state()
    }
}

impl Printer3D<ErrorState> {
    pub fn reset(self) -> Printer3D<IdleState> {
        println!("Error: reset.");
        self.into_state()
    }
}

#[cfg(test)]
mod tests {
    use crate::Printer3D;

    #[test]
    fn test_printer3d() {
        let printer = Printer3D::new();
        let printer = printer.start();
        let _ = match printer.print() {
            Ok(printer) => printer.retrieve(),
            Err(printer) => printer.reset(),
        };
    }
}
