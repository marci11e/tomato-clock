# tomato-clock
A minimalist pomodoro, with no buttons and a transparent background.

The gui is powered by [iced](https://github.com/iced-rs/iced).

[![example_img](https://github.com/marci11e/tomato-clock/blob/main/img/example.png)](https://github.com/marci11e/tomato-clock/blob/main/img/example.png)

## Usage
Use the toml configuration file to set colors, reminder, etc. tomato-clock will look for 'tomato.toml' in the startup directory, and if it doesn't find it, it will use the default configuration, whose reminder is text. 

If the image in the config file does not open, the reminder will just be simply transparent and blank. 

If the configuration file has extra fields or error fields it will not open successfully. To see the error messages you can start it from the command line. 

All fields are optional except for the mandatory fields marked in the example toml file. Missing fields will use the default configuration. 

A sample configuration file is [tomato.toml](https://github.com/marci11e/tomato-clock/blob/main/assets/tomato.toml)

#### function keys
- `r` reset timer
- `m` switching mode forward/reverse time
- `space` pause/resume countdown
- `[` decreases the countdown time in countdown mode
- `]` increase the countdown time in countdown mode
- `t` switch text color
- `b` switch background color
- `esc` exit the program
