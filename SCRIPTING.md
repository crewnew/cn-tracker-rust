# Documentation for the Scripting Language

The essential principle of the language is that you use the **Get Statements** to gain access to the data you need, then use the **Essential Statements** to build a behaviour based on the data you got access to.

## Essential Statements

- `EVERY` -> Used at the beginning of the script to determine how often the script is to be executed. Accepts `MILLISECONDS`, `SECONDS`, `MINUTES` and `HOURS`.
- `END` -> Used to close `IF`, `ELSEIF`, `ELSE` and `ITERATE` blocks.
- `IF` -> Used to build if blocks.
- `ELSEIF` -> Used inside `IF` statements to build else if blocks.
- `ELSE` -> Used inside `IF` statements to build an else block.
- `ITERATE` -> Used to iterate over data with multiple elements and execute script for each element.
- `SAVE_TO_DB` -> Used to save all the data that's been accessed through the **Get Statements** to the database.

## Conditional Statements

These statements are meant to be used with an `IF` or `ELSEIF` statement.

- `NOT` -> Used to accept a negative value as positive and thus execute the conditional statement.
- `OR` -> Used to add an or condition.
- `BIGGER` -> Used to see if a value is bigger than the other.
- `LESSER` -> Used to see if a value is lesser than the other.
- `EQ` -> Used to see if a value is equal to the other.
- `IN` -> Used to see if a value is inside a list.
- `MATCH` `Regex` -> Used to execute a regex on a variable.

## Get Statements

Once you call these statements they'll give you access to the corresponding variable that you can use inside the language.

- `GET_NETWORK_SSID` -> `NETWORK_SSID`
- `GET_PERIPHERALS` -> (`KEYSTROKES`, `MOUSE_CLICKS`)
- `GET_WINDOWS` -> `WINDOWS`, usable with `ITERATE` function and inside the `ITERATE` statement it gives you access to `TITLE`, `PROCESS_NAME`, `CMD`, `EXE`, `CWD`, `MEMORY`, `STATUS`, `START_TIME`.

## Miscellaneous

- `PRINT` -> Used to print any variable to the console.
- `CAPTURE_SCREEN` -> Captures the screen with the specified index or captures all screens. Accepts an `"INDEX"` or `"ALL"`.

## Examples

```
EVERY 5 SECONDS

GET_PERIPHERALS
GET_WINDOWS

IF KEYSTROKES BIGGER "15"
  PRINT "KEYSTROKES are bigger than 15"
END

ITERATE WINDOWS
  PRINT TITLE

  IF TITLE EQ "some title"
    PRINT "IF statement executed"

    ELSEIF TITLE EQ "some other title"
      PRINT "ELSEIF statement executed"
    END

    ELSE
      PRINT "ELSE statement executed"
    END

  END

END
```
