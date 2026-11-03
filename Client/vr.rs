use std::thread;
use std::time::Duration;

use openvr::{tracked_device_index, ApplicationType, Context, InitError, System, TrackingUniverseOrigin};

// koblingen til steam vr

pub struct Vr {
    _context: Context,
    system: System,
}

impl Vr {
    fn try_connect() -> Result<Vr, InitError> {
        // unsafe: siden vi bruker cpp bibliotek og da kan ting være feil

        let context = unsafe { openvr::init(ApplicationType::Background) }?;

        let system = context.system()?;

        Ok(Vr { _context: context, system })
    }

    pub fn wait_for_steamvr() -> Vr {
        let mut are_we_waiting = false;

        loop {
            match Vr::try_connect() {
                Ok(vr) => {
                    println!("Connected to SteamVR");
                    return vr;
                }
                Err(error)
                if error == InitError::Init_NoServerForBackgroundApp
                    || error == InitError::Init_HmdNotFound =>
                    {
                        if !are_we_waiting {
                            println!("Waiting for SteamVR... ({error})");
                            are_we_waiting = true;
                        }
                        thread::sleep(Duration::from_secs(1));
                    }

                // Permanent errors: no point in retrying
                Err(error) => {
                    eprintln!("Could not start OpenVR: {error}");
                    std::process::exit(1);
                }
            }
        }
    }
    pub fn headset_matrix(&self) -> Option<[[f32; 4]; 3]> {
        let poses = self.system.device_to_absolute_tracking_pose(
            TrackingUniverseOrigin::Standing,
            0.0,
        );

        let headset = &poses[tracked_device_index::HMD.0 as usize];

        if !headset.pose_is_valid(){
            return None;
        }

        Some(*headset.device_to_absolute_tracking())
    }


}