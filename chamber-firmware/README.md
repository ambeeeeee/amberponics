# Chamber Firmware
This is the firmware I use for my growing chamber monitoring device.

It collects information on:
- Humidity
- Temperature
- Oxygen Content

It also operates my chamber's grow lights using an RJ11 port.

The device is implemented within a junction box, since it's got decent sized holes and you can get them cheap and sturdy.

## Applied Components

These are the components I am currently using with the live system.  
In the future there would be schematics and that sort of cool stuff, but I'm still developing the device.

### Sensors
- Atlas Scientific EZO-O2™ Embedded Oxygen Sensor
- Atlas Scientific EZO-HUM™ Embedded Humidity Probe

### Communication
- XBee 3
- RJ11 6-Pin Connector
  - Controlled with PWM by the Microcontroller

### Microcontroller
- stm32h7b0