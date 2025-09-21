module sensor_track::main {
    use std::string::{String};

    public struct SensorMetadata has store, copy, drop {
        sensor_id: u8,
        location: String,
        battery_level: u8,
    }

    public fun create_metadata(
        sensor_id: u8,
        location: String,
        battery_level: u8,
    ): SensorMetadata {
        SensorMetadata {
            sensor_id,
            location,
            battery_level,
        }
    }

    public fun get_sensor_id(metadata: &SensorMetadata): u8 {
        metadata.sensor_id
    }

    public fun get_location(metadata: &SensorMetadata): String {
        metadata.location
    }

    public fun get_battery_level(metadata: &SensorMetadata): u8 {
        metadata.battery_level
    }

    public fun get_all_metadata(metadata: &SensorMetadata): (u8, String, u8) {
        (
            metadata.sensor_id,
            metadata.location,
            metadata.battery_level,
        )
    }
}
