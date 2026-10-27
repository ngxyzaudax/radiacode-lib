mod cursor;
mod decode;
mod records;
mod selection;

fn header(seq: u8, entity: u8, group: u8, ticks: i32) -> Vec<u8> {
    [vec![seq, entity, group], ticks.to_le_bytes().to_vec()].concat()
}

fn real_time(seq: u8, cps: f32, rh: f32) -> Vec<u8> {
    [
        header(seq, 0, 0, i32::from(seq) * 100),
        cps.to_le_bytes().to_vec(),
        rh.to_le_bytes().to_vec(),
        vec![10, 0, 20, 0, 0, 0, 0],
    ]
    .concat()
}

fn raw(seq: u8, cps: f32, rh: f32) -> Vec<u8> {
    [header(seq, 0, 1, 0), cps.to_le_bytes().to_vec(), rh.to_le_bytes().to_vec()].concat()
}

fn rare(seq: u8, battery_percent: f32, temperature_c: f32) -> Vec<u8> {
    let temperature_raw = (temperature_c * 100.0 + 2000.0) as u16;
    let charge_raw = (battery_percent * 100.0) as u16;
    [
        header(seq, 0, 3, 0),
        600u32.to_le_bytes().to_vec(),
        0.0001f32.to_le_bytes().to_vec(),
        temperature_raw.to_le_bytes().to_vec(),
        charge_raw.to_le_bytes().to_vec(),
        0u16.to_le_bytes().to_vec(),
    ]
    .concat()
}

fn event(seq: u8, code: u8, param: u8, flags: u16) -> Vec<u8> {
    [header(seq, 0, 7, 0), vec![code, param], flags.to_le_bytes().to_vec()].concat()
}

fn realtime(seq: u8) -> Vec<u8> {
    real_time(seq, 5.0, 0.001)
}
