# passphrs

passphrs is a cross-platform CLI tool to generate a diceware-style passphrase with customizable parameters.

In standard operation passphrs will copy the password to the clipboard and then clear the clipboard after 10 seconds. This behaviour can be changed, and passphrs can instead display the passphrase in the terminal along with its entropy. Doing so will leave the passphrase in your console's log so is not recommended for actual use.

By default, passphrs generates a 7-word passphrase using the EFF Large Wordlist for Passphrases, capitalized, separated by spaces, and with no additional characters. This can be customized by using a custom wordlist, changing the passphrase length, changing the separator, changing the capitalization, or adding 'salt' in the form of a fixed number of random characters added to the end of a random word in the passphrase.

This requires the X11 library on Linux to access the clipboard.
