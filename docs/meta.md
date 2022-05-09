## TODO

* [x] Add internal **read from bytestream** method for virtual data.
* [x] Test line delimiter
* [x] Make drop-in replacement for current ced's implementation
* [ ] Make virtual data's api more ergonomic for library usage
* [ ] Test various formats of csv data

## How csv reader is different from ced?

Actually this project was made to move a burden of csv parsing from ced to
separate crate.

But the thing is ced uses virtual data structure which incorporates value
limiter with values. So that value has some kind of types. Also virtual data assumes there can be editing which is not necessarily a pure purpose of csv\_reader
