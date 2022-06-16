## TODO

* [ ] Remove unnecessary reference of strings
	* [ ] If internal method, don't make it generic if possible
* [x] Vritual Array : This is mostly for CED's ergonomics.
	* [x] Reader can read as array(VirtualArray) 
	* [ ] Should check if things work. Test it in ced.
* [x] Exposed reader\_option 
* [x] Enabled number columns, though number columns cannot be selected with
names.
* [x] Fixed a typo in reader's error message
* [x] Better docs.rs documentation
* [x] Custom header
* [x] Clippied the source code
* [x] Make virtual data's api more ergonomic for library usage

* [ ] Test various formats of csv data

## How csv reader is different from ced?

Actually this project was made to move a burden of csv parsing from ced to
separate crate.

But the thing is ced uses virtual data structure which incorporates value
limiter with values. So that value has some kind of types. Also virtual data assumes there can be editing which is not necessarily a pure purpose of csv\_reader

## DONE

* [x] Add internal **read from bytestream** method for virtual data.
* [x] Test line delimiter
* [x] Make drop-in replacement for current ced's implementation
* [x] Fixed a bug where arbitrary column was infinitely added, at least the

program tried but failed of course.
* [x] Added a new option for ignore empty row.
* [x] Support complete spec of csv values : Kind of?
	* [x] newline inside double quotes are allowed... For real...
* [x] Fixed missing parser option for custom line ending
* [x] Made rename-column doesn't let number column for sanity

* [x] PartialOrd for Value
* [x] Docs.rs compatible documentations.
* [x] Trim reader option
* [x] ReadOnly Struct for better usability
