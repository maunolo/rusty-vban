use std::{mem::MaybeUninit, sync::Arc};

use cpal::{Sample, SupportedStreamConfig};
use ringbuf::{Consumer, HeapRb, Producer, SharedRb};

pub type VbanStreamConsumer = Consumer<i16, Arc<SharedRb<i16, Vec<MaybeUninit<i16>>>>>;
pub type VbanStreamProducer = Producer<i16, Arc<SharedRb<i16, Vec<MaybeUninit<i16>>>>>;

pub fn start_buffer(
    latency: f32,
    config: &SupportedStreamConfig,
) -> (VbanStreamProducer, VbanStreamConsumer) {
    let latency_frames = (latency / 1_000.0) * config.sample_rate().0 as f32;
    let latency_samples = latency_frames as usize * config.channels() as usize;

    let ring = HeapRb::<i16>::new(latency_samples * 2);
    let (mut producer, consumer) = ring.split();

    // Fill the samples with 0.0 equal to the length of the delay.
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        producer.push(0.0.to_sample::<i16>()).ok();
    }

    (producer, consumer)
}
