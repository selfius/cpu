# GATE Level CPU emulator

A very much work-in-progress attempt at building a full CPU from nothing but NAND Gates.

To make thing more understandable and eaiser to maintain most components of the CPU
are described with a simple visual-ish programming:

Example of NOT gate definition:
```
           ┏━━━━┓
        ─┬─┨nand┠──
         └─┨    ┃
           ┗━━━━┛
```

A more complex example of a 2 inputs to 4 outputs decoder:
```

         ┏━━━┓       ┏━━━┓
      ──┬┨not┠─────┬─┨   ┃
        │┗━━━┛┏━━━┓│ ┃and┠─
      ──┼┬────┨not┠┼┬┨   ┃
        ││    ┗━━━┛││┗━━━┛
        ││         ││┏━━━┓
        ││         └┼┨and┠─
        │├──────────┼┨   ┃
        ││          │┗━━━┛
        ││          │┏━━━┓
        ├┼──────────┼┨and┠─
        ││          └┨   ┃
        ││           ┗━━━┛
        ││           ┏━━━┓
        └┼───────────┨and┠─
         └───────────┨   ┃
                     ┗━━━┛
```

## Drawing all the fancy diagrams

While this way of describing circuits makes it easier to understand the connections between components it definitely does not make it all that easier to code them (at least the act of typing them out)

To help with there's a simple [nvim helper script](https://github.com/selfius/dotfiles/blob/master/.config/nvim/lua/localplugins/boxes/plugin/boxes.lua). While it's still full of bugs it mostly gets the job done:
![demo](https://raw.githubusercontent.com/selfius/cpu/b1060703aa9edbb93ca519a820508bc3e9c9de63/static/demo.gif)

