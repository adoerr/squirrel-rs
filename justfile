mod device 'device'
mod host 'host'

# Remove build artifacts and disk image
@clean: device::clean host::clean

# Check formatting and linting
@check: device::check host::check

# Sort dependencies
@sort: device::sort host::sort

# Update dependencies
@update: device::update host::update

# Check for outdated dependencies
@outdated: device::outdated host::outdated

# Release build all components
@release: device::release host::release

# Debug build all components
@debug: device::debug host::debug

@flash binary:
    cd device && cargo run --bin {{binary}}

@attach binary:
    probe-rs attach --chip RP2040 device/target/thumbv6m-none-eabi/debug/{{binary}}

@run binary:
    cd host && RUST_LOG=info cargo run --bin {{binary}}
