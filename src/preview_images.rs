use std::{cell::RefCell, collections::BTreeMap};
use telorgon::{ImageId, shell::capture::CaptureSource};

/// A resource belongs to a source incarnation, never its position in a snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PreviewImages(RefCell<BTreeMap<(CaptureSource, u64), ImageId>>);

impl PreviewImages {
    pub fn id(&self, source: CaptureSource, epoch: u64) -> ImageId {
        let mut images = self.0.borrow_mut();
        let next = ImageId(
            0x7000_0000u32
                .checked_add(images.len().try_into().unwrap())
                .unwrap(),
        );
        *images.entry((source, epoch)).or_insert(next)
    }
}
