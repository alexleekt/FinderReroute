use axuielement::prelude::*;

fn main() {
    env_logger::init();
    
    let system = system_wide().expect("system-wide accessibility not available");
    
    // Scan along the left edge where the Dock is typically located
    for y in (50..=600).step_by(50) {
        match system.element_at_position(50.0, y as f32) {
            Ok(Some(el)) => {
                let title = el.string_attribute(axuielement::ax_attribute::AX_TITLE_ATTRIBUTE).unwrap_or(None);
                let subrole = el.string_attribute(axuielement::ax_attribute::AX_SUBROLE_ATTRIBUTE).unwrap_or(None);
                let role = el.string_attribute(axuielement::ax_attribute::AX_ROLE_ATTRIBUTE).unwrap_or(None);
                println!("pos=(50, {:3}): title={:12?} subrole={:30?} role={:?}", y, title, subrole, role);
            }
            Ok(None) => {
                println!("pos=(50, {:3}): no element", y);
            }
            Err(e) => {
                println!("pos=(50, {:3}): error={:?}", y, e);
            }
        }
    }
}
