# 0.2.0

## Breaking changes

- Changed method names : now "Reader" invokes "data\_from\_stream" not "read\_from\_stream"
- Get\_cell now returns option and don't panic on invalid cooridnate.

## ETC

- FET : VirtualArray for raw editing of csv values.
- FET : "to\_vec" for row
- FET : New "custom\_header" method for reader
- CHG : Changed "drop" method to "drop\_data"
- ERG : More documentations for docs.rs
- ERG : Applied clippy fix
- BUG : Row count typo

# 0.1.4

- NEW : ReadOnlyData, ReadOnlyDataRef for data iteration

# 0.1.3

- PartialOrd derive for "Value" struct
- Docs.rs compatible documentations
- Fixed that lib.rs and readme example was not working.
- Trim reader option
