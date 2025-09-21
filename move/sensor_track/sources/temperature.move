module sensor_track::temperature {
    use std::string::{String};
    use iota::event;
    use sensor_track::main::{Self, SensorMetadata};

    public struct TemperatureReading has key, store {
        id: UID,
        metadata: SensorMetadata,
        temperature_celsius: u32,
    }

    public struct TemperatureRecorded has copy, drop {
        sensor_id: u8,
        reading_id: ID,
        temperature_celsius: u32,
    }

    public fun push_reading(
        sensor_id: u8,
        location: String,
        battery_level: u8,
        temperature_celsius: u32,
        ctx: &mut TxContext
    ) {
        let metadata = main::create_metadata(sensor_id, location, battery_level);
        let reading = TemperatureReading {
            id: object::new(ctx),
            metadata,
            temperature_celsius,
        };

        let reading_id = object::id(&reading);

        event::emit(TemperatureRecorded {
            sensor_id,
            reading_id,
            temperature_celsius,
        });

        transfer::transfer( reading, ctx.sender())
    }

    public fun get_temperature(reading: &TemperatureReading): u32 {
        reading.temperature_celsius
    }

    public fun get_metadata(reading: &TemperatureReading): &SensorMetadata {
        &reading.metadata
    }

    public fun get_sensor_id(reading: &TemperatureReading): u8 {
        main::get_sensor_id(&reading.metadata)
    }

    public fun get_all_data(reading: &TemperatureReading): (u8, String, u8,  u32) {
        let (sensor_id, location, battery_level) = main::get_all_metadata(&reading.metadata);
        (sensor_id, location, battery_level,  reading.temperature_celsius)
    }
}