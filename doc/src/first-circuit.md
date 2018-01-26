# First Circuit

TODO

```
component R {
    prefix = "R";
    footprint = "";

    pin A: passive = 1;
    pin B: passive = 2;
}

component LED {
    prefix = "D";
    footprint = "";

    pin anode: passive = 1;
    pin cathode: passive = 2;
}

component PinHeader1x2 {
    prefix = "J";
    footprint = "";

    pin "5V0": power_out = 1;
    pin GND: power_out = 2;
}

abstract component Main {
    net "5V0", GND;
    net R_to_LED;

    PinHeader1x2 {
        value = "Power";
        "5V0": "5V0";
        GND: GND;
    }

    R {
        A: "5V0";
        B: R_to_LED;
    }

    LED {
        anode: R_to_LED;
        cathode: GND;
    }
}
```
