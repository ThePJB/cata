use crate::kmath::*;

use cpal::Stream;
use cpal::traits::*;
use ringbuf::*;

pub struct Audio {
    stream: Stream,
    channel: Producer<SoundCommand>,
}

impl Audio {
    pub fn new() -> Audio {
        let rb = RingBuffer::<SoundCommand>::new(50);
        let (mut prod, mut cons) = rb.split();
        let mut audio = Audio {
            stream: stream_setup_for(sample_next, cons).expect("no can make stream"),
            channel: prod,
        };
        audio.stream.play().expect("cant play audio stream");
        audio
    }

    pub fn handle_command(&mut self, sc: SoundCommand) {
        self.channel.push(sc);
    }
}

pub fn db_to_vol(db: f32) -> f32 {
    10.0f32.powf(0.05 * db)
}

pub fn vol_to_db(vol: f32) -> f32 {
    20.0f32 * vol.log10()
}

#[derive(Clone, Copy, Debug)]
pub struct SoundDesc {
    pub f: f32,
    pub n: f32,
    pub troll: f32,
    pub ea: f32,
    pub ed: f32,
    pub es: f32,
    pub er: f32,
    pub detune: f32,
    pub voices: f32,
    pub amp: f32,
    pub cut: f32,
    pub cur: f32,
    pub cdt: f32,
    pub cdr: f32,
    pub aout: f32,
    pub release: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SoundCommand {
    pub sd: SoundDesc,
    pub id: u32,
}

pub struct Channel {
    pub sd: SoundDesc,
    pub age: f32,
    pub phases: Vec<f32>,
    pub id: u32,
}

impl Channel {
    pub fn tick(&mut self) -> f32 {
        self.age += 1.0 / 44100.0;

        let sd = self.sd;

        let voices_len = sd.voices.floor() as usize;
        let n_len = sd.n.floor() as usize;
        self.phases.resize(voices_len*n_len, 0.0);  // or resize with random phases, is better

        // pre compression
        let mut acc = 0.0;

        let a_vol = db_to_vol(sd.amp);

        let a_env = 1.0;

        let a_voices = 1.0 / voices_len as f32;

        for detune_voice_num in 0..voices_len {
            for n in 0..n_len {
                let a_roll = 1.0 / ((n+1) as f32).powf(sd.troll);

                let detune_interval = 2.0f32.powf(sd.detune / 1200.0);
                let f = sd.f * (n + 1) as f32;
                let f = f * detune_interval.powf(detune_voice_num as f32);

                let idx = detune_voice_num * n_len + n;
                self.phases[idx] = (self.phases[idx] + f / 44100.0).fract();
                acc += a_voices * a_env * a_roll * a_vol * (2.0 * PI * self.phases[idx]).sin();
            }
        }


        // now do compression
        // change db value or amplitude value?
        let comp = {
            let cut_vol = db_to_vol(sd.cut);
            let cdt_vol = db_to_vol(sd.cdt);
            if acc < cut_vol {
                let gain = lerp(sd.cur, 1.0, (sd.cur - acc)/sd.cur);
                gain * acc
            } else if acc > cdt_vol {
                cdt_vol + (acc - cdt_vol) / sd.cdr
            } else {
                acc
            }
        };
        let out = comp * sd.aout;
        if out > 1.0 {
            1.0
        } else if out < -1.0 {
            -1.0
        } else {
            out
        }
    }

    pub fn should_remove(&self) -> bool {
        self.sd.release && self.age > self.sd.er
    }
}

#[derive(Default)]
pub struct Mixer {
    pub sample_count: u64,
    pub channels: Vec<Channel>,
}

impl Mixer {
    pub fn handle_command(&mut self, sc: SoundCommand) {
        println!("handle command {:?}", sc);
        for i in 0..self.channels.len() {
            // if its already playing we may want to blend
            if self.channels[i].id == sc.id {

                self.channels[i].sd = sc.sd;

                self.channels[i].age = 0.0;
                return;
            }
        }
        let voices_len = sc.sd.voices.floor() as usize;
        let n_len = sc.sd.n.floor() as usize;
        self.channels.push(Channel {
            sd: sc.sd,
            id: sc.id,
            age: 0.0,
            phases: vec![0.0; voices_len*n_len], // todo preallocate max phases for detune
        });
    }

    pub fn tick(&mut self) -> f32 {
        self.sample_count += 1;

        let mut i = self.channels.len();
        if i == 0 { return 0.0 }
        i -= 1;
        let mut acc = 0.0;
        loop {
            acc += self.channels[i].tick();
            
            if self.channels[i].should_remove() { 
                println!("removing {}", i);
                self.channels.swap_remove(i);
            }

            if i == 0 { break; }
            i -= 1;
        }
        acc
    }
}



// pub fn env_amplitude(a: f32, d: f32, s: f32, r: f32, curr_sample: u32, sample_rate: u32, released_sample: Option<u32>) -> f32 {
//     // +1 for useful recursion
//     let A = self.a * sample_rate as f32;
//     let D = self.d * sample_rate as f32;
//     let S = self.s;
//     let R = self.r * sample_rate as f32;

//     if let Some(released_on) = released_sample {
//         let num_released = curr_sample - released_on;
//         let release_value = self.amplitude(released_on, sample_rate, None);
//         return lerp(release_value, 0.0, num_released as f32 / R).max(0.0);
//     }
//     if curr_sample as f32 <= A {
//         return lerp(0.0, 1.0, curr_sample as f32 / A);
//     }
//     if curr_sample as f32 <= D + A {
//         return lerp(1.0, S, (curr_sample as f32 - A)/D);
//     }
//     S
// }




fn sample_next(o: &mut SampleRequestOptions) -> f32 {
    o.mixer.tick()
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub nchannels: usize,

    // pub filter: Filter,

    pub mixer: Mixer,

    pub channel: Consumer<SoundCommand>,
}

pub fn stream_setup_for<F>(on_sample: F, channel: Consumer<SoundCommand>) -> Result<cpal::Stream, anyhow::Error>
where
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => stream_make::<f32, _>(&device, &config.into(), on_sample, channel),
        cpal::SampleFormat::I16 => stream_make::<i16, _>(&device, &config.into(), on_sample, channel),
        cpal::SampleFormat::U16 => stream_make::<u16, _>(&device, &config.into(), on_sample, channel),
    }
}

pub fn host_device_setup(
) -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.name()?);

    let config = device.default_output_config()?;
    println!("Default output config : {:?}", config);

    Ok((host, device, config))
}

pub fn stream_make<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    on_sample: F,
    channel: Consumer<SoundCommand>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static + Copy,
{
    let sample_rate = config.sample_rate.0 as f32;
    let nchannels = config.channels as usize;
    let mut request = SampleRequestOptions {
        sample_rate,
        nchannels,

        mixer: Mixer::default(),

        channel,
    };
    let err_fn = |err| eprintln!("Error building output sound stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            on_window(output, &mut request, on_sample)
        },
        err_fn,
    )?;

    Ok(stream)
}

fn on_window<T, F>(output: &mut [T], request: &mut SampleRequestOptions, mut on_sample: F)
where
    T: cpal::Sample,
    F: FnMut(&mut SampleRequestOptions) -> f32 + std::marker::Send + 'static,
{
    if let Some(sc) = request.channel.pop() {
        request.mixer.handle_command(sc);
    }
    for frame in output.chunks_mut(request.nchannels) {
        let value: T = cpal::Sample::from::<f32>(&on_sample(request));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}