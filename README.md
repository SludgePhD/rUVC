Leap Motion's Extension Unit claims to have 0 controls, but their librealuvc fork sets them anyways
(among other things in order to turn on the LEDs).

just look at https://github.com/leapmotion/leapuvc/blob/master/LeapUVC-Manual.pdf

`GET_DEF` and `GET_LEN` on the Video Streaming interface seem to just hang.

Leap Motion uses the "Contrast" control on the Processing Unit for custom controls.
Value format: `0V000LLL`
If `V` is set, the feature identified by `LLL` is turned on, otherwise it is turned off.
`LLL` can be `010`, `011`, or `100` to identify the 3 LEDs. It can also be `000` to control the
"HDR" feature (not sure what that is).

leapd sets "Contrast" to the following values, in order:
- `0x0001` (as part of setting *all* controls)
- `0x0000` -> Turn off HDR
- `0x0042` / `0b01000010` -> Turn on LED 1
- `0x0043` / `0b01000011` -> Turn on LED 2
- `0x0044` / `0b01000100` -> Turn on LED 3
- `0x0006` / `0b00000110` -> Turn off ???
- `0x3C05` / `0b00111100_00000101` -> ?????

The "Exposure" control seems to be remapped to the UVC "Zoom (Absolute)" control.

The "Saturation" and "Sharpness" controls are repurposed to access the "opaque calibration" data of
the device: *Sharpness* is set to an address in range `100..=255`, then *Saturation* is read to
obtain the byte stored at that offset.

Example result, including the <100 addresses:

```
calibration data: [4c, 50, 35, 33, 33, 30, 34, 31, 37, 32, 35, 34, 31, 0, 70, 73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9c, 39, ab, 1f, 55, 1e, e6, 8e, a0, dc, e8, 44, 64, db, 56, 82, 8, 9, 11, 0, 43, 41, 0, 87, 50, 6a, 4c, 51, dd, 5a, 20, 42, 66, c, 29, 3d, bb, e7, 5, 43, 27, 9c, e, bf, 3f, 9d, 89, 40, 15, 88, 32, 3a, b1, a2, 5a, bb, e5, d0, a2, 3f, ac, ad, 98, 3e, 97, ff, 90, 3c, 5a, 64, bb, 3f, e9, b7, af, 3e, 9e, b5, db, bb, 58, 5d, 3, 43, 34, 21, 2a, 40, 78, df, 9c, 40, 65, cb, b6, b9, c6, fe, 73, bb, 6e, fa, 85, bb, 3a, ee, 5, 43, 61, 9a, 70, 3f, 9a, 84, ca, 40, f6, 49, e, 3a, dc, ac, 8c, bb, e5, d0, a2, 3f, ac, ad, 98, 3e, 97, ff, 90, 3c, 5a, 64, bb, 3f, e9, b7, af, 3e, 9e, b5, db, bb, 58, 5d, 3, 43, 49, 2, 8a, 40, 78, df, 9c, 40, 1c, a8, a5, bb, a3, 81, cd, ba, 30, 98, 85, 3b, 7b, 26, 62, 9d]
```

First non-zero block spells "LP53304172541" in ASCII, rest isn't ASCII. Could be or contain the firmware version/ID (this is on 1.7.0).

```
Bus 001 Device 028: ID f182:0003 Leap Motion Controller
Device Descriptor:
  bLength                18
  bDescriptorType         1
  bcdUSB               2.00
  bDeviceClass          239 Miscellaneous Device
  bDeviceSubClass         2
  bDeviceProtocol         1 Interface Association
  bMaxPacketSize0        64
  idVendor           0xf182 Leap Motion
  idProduct          0x0003 Controller
  bcdDevice            0.00
  iManufacturer           1 Leap Motion
  iProduct                2 Leap Motion Controller
  iSerial                 0
  bNumConfigurations      1
  Configuration Descriptor:
    bLength                 9
    bDescriptorType         2
    wTotalLength       0x0163
    bNumInterfaces          2
    bConfigurationValue     1
    iConfiguration          0
    bmAttributes         0x80
      (Bus Powered)
    MaxPower              500mA
    Interface Association:
      bLength                 8
      bDescriptorType        11
      bFirstInterface         0
      bInterfaceCount         2
      bFunctionClass         14 Video
      bFunctionSubClass       3 Video Interface Collection
      bFunctionProtocol       0
      iFunction               0
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       0
      bNumEndpoints           1
      bInterfaceClass        14 Video
      bInterfaceSubClass      1 Video Control
      bInterfaceProtocol      0
      iInterface              0
      VideoControl Interface Descriptor:
        bLength                13
        bDescriptorType        36
        bDescriptorSubtype      1 (HEADER)
        bcdUVC               1.00
        wTotalLength       0x0050
        dwClockFrequency        0.001000MHz
        bInCollection           1
        baInterfaceNr( 0)       1
      VideoControl Interface Descriptor:
        bLength                18
        bDescriptorType        36
        bDescriptorSubtype      2 (INPUT_TERMINAL)
        bTerminalID             2
        wTerminalType      0x0201 Camera Sensor
        bAssocTerminal          0
        iTerminal               0
        wObjectiveFocalLengthMin      0
        wObjectiveFocalLengthMax      0
        wOcularFocalLength            0
        bControlSize                  3
        bmControls           0x00000228
          Exposure Time (Absolute)
          Focus (Absolute)
          Zoom (Absolute)
      VideoControl Interface Descriptor:
        bLength                12
        bDescriptorType        36
        bDescriptorSubtype      5 (PROCESSING_UNIT)
      Warning: Descriptor too short
        bUnitID                 5
        bSourceID               2
        wMaxMultiplier          0
        bControlSize            3
        bmControls     0x0000027b
          Brightness
          Contrast
          Saturation
          Sharpness
          Gamma
          White Balance Temperature
          Gain
        iProcessing             0
        bmVideoStandards     0x1c
          PAL - 625/50
          SECAM - 625/50
          NTSC - 625/50
      VideoControl Interface Descriptor:
        bLength                28
        bDescriptorType        36
        bDescriptorSubtype      6 (EXTENSION_UNIT)
        bUnitID                 6
        guidExtensionCode         {ffffffff-ffff-ffff-ffff-ffffffffffff}
        bNumControls            0
        bNrInPins               1
        baSourceID( 0)          5
        bControlSize            3
        bmControls( 0)       0x00
        bmControls( 1)       0x00
        bmControls( 2)       0x00
        iExtension              0
      VideoControl Interface Descriptor:
        bLength                 9
        bDescriptorType        36
        bDescriptorSubtype      3 (OUTPUT_TERMINAL)
        bTerminalID             3
        wTerminalType      0x0101 USB Streaming
        bAssocTerminal          0
        bSourceID               6
        iTerminal               0
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x82  EP 2 IN
        bmAttributes            3
          Transfer Type            Interrupt
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0040  1x 64 bytes
        bInterval               8
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        1
      bAlternateSetting       0
      bNumEndpoints           1
      bInterfaceClass        14 Video
      bInterfaceSubClass      2 Video Streaming
      bInterfaceProtocol      0
      iInterface              0
      VideoStreaming Interface Descriptor:
        bLength                            14
        bDescriptorType                    36
        bDescriptorSubtype                  1 (INPUT_HEADER)
        bNumFormats                         1
        wTotalLength                   0x00dd
        bEndpointAddress                 0x83  EP 3 IN
        bmInfo                              0
        bTerminalLink                       3
        bStillCaptureMethod                 0
        bTriggerSupport                     0
        bTriggerUsage                       0
        bControlSize                        1
        bmaControls( 0)                     0
      VideoStreaming Interface Descriptor:
        bLength                            27
        bDescriptorType                    36
        bDescriptorSubtype                  4 (FORMAT_UNCOMPRESSED)
        bFormatIndex                        1
        bNumFrameDescriptors                6
        guidFormat                            {32595559-0000-0010-8000-00aa00389b71}
        bBitsPerPixel                      16
        bDefaultFrameIndex                  4
        bAspectRatioX                       0
        bAspectRatioY                       0
        bmInterlaceFlags                 0x00
          Interlaced stream or variable: No
          Fields per frame: 2 fields
          Field 1 first: No
          Field pattern: Field 1 only
        bCopyProtect                        0
      VideoStreaming Interface Descriptor:
        bLength                            30
        bDescriptorType                    36
        bDescriptorSubtype                  5 (FRAME_UNCOMPRESSED)
        bFrameIndex                         1
        bmCapabilities                   0x00
          Still image unsupported
        wWidth                            640
        wHeight                           480
        dwMinBitRate                282624000
        dwMaxBitRate                282624000
        dwMaxVideoFrameBufferSize      614400
        dwDefaultFrameInterval         173913
        bFrameIntervalType                  1
        dwFrameInterval( 0)            173913
      VideoStreaming Interface Descriptor:
        bLength                            30
        bDescriptorType                    36
        bDescriptorSubtype                  5 (FRAME_UNCOMPRESSED)
        bFrameIndex                         2
        bmCapabilities                   0x00
          Still image unsupported
        wWidth                            640
        wHeight                           240
        dwMinBitRate                282624000
        dwMaxBitRate                282624000
        dwMaxVideoFrameBufferSize      307200
        dwDefaultFrameInterval          86956
        bFrameIntervalType                  1
        dwFrameInterval( 0)             86956
      VideoStreaming Interface Descriptor:
        bLength                            30
        bDescriptorType                    36
        bDescriptorSubtype                  5 (FRAME_UNCOMPRESSED)
        bFrameIndex                         3
        bmCapabilities                   0x00
          Still image unsupported
        wWidth                            640
        wHeight                           120
        dwMinBitRate                262963200
        dwMaxBitRate                262963200
        dwMaxVideoFrameBufferSize      153600
        dwDefaultFrameInterval          46728
        bFrameIntervalType                  1
        dwFrameInterval( 0)             46728
      VideoStreaming Interface Descriptor:
        bLength                            30
        bDescriptorType                    36
        bDescriptorSubtype                  5 (FRAME_UNCOMPRESSED)
        bFrameIndex                         4
        bmCapabilities                   0x00
          Still image unsupported
        wWidth                            752
        wHeight                           480
        dwMinBitRate                288768000
        dwMaxBitRate                288768000
        dwMaxVideoFrameBufferSize      721920
        dwDefaultFrameInterval         200000
        bFrameIntervalType                  1
        dwFrameInterval( 0)            200000
      VideoStreaming Interface Descriptor:
        bLength                            30
        bDescriptorType                    36
        bDescriptorSubtype                  5 (FRAME_UNCOMPRESSED)
        bFrameIndex                         5
        bmCapabilities                   0x00
          Still image unsupported
        wWidth                            752
        wHeight                           240
        dwMinBitRate                288768000
        dwMaxBitRate                288768000
        dwMaxVideoFrameBufferSize      360960
        dwDefaultFrameInterval         100000
        bFrameIntervalType                  1
        dwFrameInterval( 0)            100000
      VideoStreaming Interface Descriptor:
        bLength                            30
        bDescriptorType                    36
        bDescriptorSubtype                  5 (FRAME_UNCOMPRESSED)
        bFrameIndex                         6
        bmCapabilities                   0x00
          Still image unsupported
        wWidth                            752
        wHeight                           120
        dwMinBitRate                274329600
        dwMaxBitRate                274329600
        dwMaxVideoFrameBufferSize      180480
        dwDefaultFrameInterval          52631
        bFrameIntervalType                  1
        dwFrameInterval( 0)             52631
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x83  EP 3 IN
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0200  1x 512 bytes
        bInterval               1
Device Qualifier (for other device speed):
  bLength                10
  bDescriptorType         6
  bcdUSB               2.00
  bDeviceClass          239 Miscellaneous Device
  bDeviceSubClass         2
  bDeviceProtocol         1 Interface Association
  bMaxPacketSize0        64
  bNumConfigurations      1
Device Status:     0x0000
  (Bus Powered)
```
