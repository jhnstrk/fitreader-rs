# fitreader-rs
Reading / writing Fit files in Rust. (Prototype)

# Logger
It uses env_logger: https://docs.rs/env_logger/latest/env_logger/

Uses RUST_LOG to set the logging level.

```shell
RUST_LOG=debug exec
```

# Messages

The FIT spec defines messages in an Excel spreadsheet.
To convert fields to JSON the following formala may be useful: here for row 5 from the messages sheet..

```
="{""field_defn_num"":"&B5&", 
  ""field_name"": """&C5&""",""field_type"":"""&D5&"""" &
   IF(ISBLANK(G5),"",",""scale"":" & G5) & 
   IF(ISBLANK(H5),"",",""offset"":" & H5) & 
   IF(ISBLANK(I5),"",",""units"":""" & I5 &"""") & 
   IF(ISBLANK(E5),"",",""array"":true") & "},"
```

Additional undocumented fields may be found in the fit4ruby project: https://github.com/scrapper/fit4ruby

# References

* https://www.thisisant.com/resources/fit
* https://developer.garmin.com/fit/overview/

