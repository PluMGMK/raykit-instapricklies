raykit-instapricklies
=====================

A patcher for the Rayman Designer EXE to allow "red" events, other than the usual small and big prickly, to be made insta-killing by changing their hitpoints.
It accomplishes this by means of simple adjustments to a function-pointer table governing the collision behaviour of different object types.
The adjustments are made in the EXE's relocation tables.

# The Issue

For over a decade, it has been known that to get a red insta-kill prickly (big or small) in Rayman Designer, like the ones from the Educational games,
you need to copy the `EVE.MLT` code for the corresponding blue one, and change:
* The `DES` from `CPL` to `CPR`, and
* The hitpoints from `0` to `1`.

So, for example, you can copy the big blue prickly's code, which looks like this:
```
/ grosse boule bleue /
def,MS_cpl_bpic,CPL, 4 ,
CPL.ETA,
 2 ,     254,
 33 , 255,
1,1,
0,3,
18,32,0,
0,0,0,
 107 , 255 , 5 ,
```
And change it to this:
```
/ grosse boule rouge /
def,MS_cpl_bpic_insta,CPR, 4 ,
CPL.ETA,
 2 ,     254,
 33 , 255,
1,1,
0,3,
18,32,0,
0,0,1,
 107 , 255 , 5 ,
```

It is possible to do something similar with moving pricklies (e.g. `MS_ouye4`) and even the big bouncy platforms (`MS_cpl_blu1`).
However, while these events will be red, they won't be insta-killing, because the code in the EXE wasn't set up for that.
This patcher solves that problem, allowing you to use insta-killing red moving pricklies and bouncy platforms.

You can also get red insta-killing prickly swings (`MS_lia_mont`) with this patcher, but in this case, you must set the hitpoints to *`2` or higher*.
The reason for this is that the stock `EVE.MLT` uses hitpoints of `1` for the stationary swing (even though there is no reason to do so).

# Usage

If you're on Windows (either 32- or 64-bit), you can grab the EXE from the latest entry on the [releases page](https://github.com/PluMGMK/raykit-instapricklies/releases).
Once you have downloaded it, you can drag your `RAYKIT.EXE` file onto this one, and it will patch it and create a `RAYKIT.EXE.BAK.IPR` backup file in case anything goes wrong.
The reason for the `.IPR` suffix is just to distinguish it from any other `RAYKIT.EXE.BAK` that you might have, e.g. from [my other patcher](https://github.com/PluMGMK/raykit_eduobjs).
Note: I've only tested this on the GOG version of Rayman Designer, so it may fail on other versions.

Alternatively, if you're comfortable with the command line and have a Rust nightly toolchain installed, you can do this to compile it yourself:
```
$ git clone https://github.com/PluMGMK/raykit-instapricklies.git
$ cd raykit-instapricklies
$ cargo run --release -- /PATH/TO/RAYKIT.EXE
```
