//! Bounded, address-bound raid timer for police that have identified a hiding flat.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaidEvent {
    Started,
    AtDoor,
    Warning,
    Breach,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApartmentRaid {
    pub apartment_id: u64,
    elapsed: f32,
    breached: bool,
}

impl ApartmentRaid {
    /// Starts only after a dispatched pursuit identifies the current apartment.
    /// Once started, the raid stays tied to that address and its clock survives exits.
    pub fn tick(
        &mut self,
        pursuit_active: bool,
        detected_apartment: Option<u64>,
        dt: f32,
    ) -> Option<RaidEvent> {
        if !pursuit_active || !dt.is_finite() || dt <= 0. {
            return None;
        }
        if self.apartment_id != detected_apartment.unwrap_or(self.apartment_id) {
            return None;
        }
        let old = self.elapsed;
        self.elapsed = (self.elapsed + dt).min(90.);
        if old < 90. && self.elapsed >= 90. && !self.breached {
            self.breached = true;
            Some(RaidEvent::Breach)
        } else if old < 80. && self.elapsed >= 80. {
            Some(RaidEvent::Warning)
        } else if old < 60. && self.elapsed >= 60. {
            Some(RaidEvent::AtDoor)
        } else {
            None
        }
    }

    pub fn begin(apartment_id: u64) -> Self {
        Self {
            apartment_id,
            elapsed: 0.,
            breached: false,
        }
    }
}

pub fn begin_if_identified(
    current: Option<ApartmentRaid>,
    pursuit_active: bool,
    apartment_id: Option<u64>,
) -> (Option<ApartmentRaid>, Option<RaidEvent>) {
    if current.is_some() || !pursuit_active {
        return (current, None);
    }
    apartment_id.map_or((None, None), |id| {
        (Some(ApartmentRaid::begin(id)), Some(RaidEvent::Started))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raid_needs_dispatched_pursuit_and_keeps_its_address() {
        let (raid, event) = begin_if_identified(None, false, Some(4));
        assert!(raid.is_none());
        assert!(event.is_none());
        let (mut raid, event) = begin_if_identified(None, true, Some(4));
        assert_eq!(event, Some(RaidEvent::Started));
        let raid = raid.as_mut().unwrap();
        assert_eq!(raid.tick(true, Some(5), 60.), None);
        assert_eq!(raid.tick(true, None, 60.), Some(RaidEvent::AtDoor));
        assert_eq!(raid.tick(true, Some(4), 20.), Some(RaidEvent::Warning));
        assert_eq!(raid.tick(true, Some(4), 10.), Some(RaidEvent::Breach));
        assert!(raid.breached);
        assert_eq!(raid.apartment_id, 4);
    }
}
