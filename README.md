# Account Transaction Reader

Simple CLI utility to read transactions from a `.csv` file. The application doesn't use async because it doesn't need to and it would be a slight overhead for the job it does. It operates on good-will meaning that any errors that it encounters while processing a transaction are ignored and the operation is not performed.<br>

The output is written to STDOUT and the program accepts/processes a precision of up to 4 fractional digits. When the fraction is unnecessary e.g. because it's only zeros, then the fraction is omitted.<br>

It uses `clap` to parse the CLI args for future extensibility, while it doesn't really need to since the only necessary argument is the path to the csv file which could also be achieved by only using the standard library.

## Testing

The most crucial piece, the `AccountService` has a couple unit tests for the edge cases that should be ignored. Besides that I provide a simple testing suite with two example files that should be possible to process without the application crashing. There is a simple file and a bigger/more complex file that was generated with ChatGPT simply to test the performance and error acceptance of the program.
