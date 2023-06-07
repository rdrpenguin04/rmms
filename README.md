# RMMS (Robust Music-Making Studio)
LMMS successor candidate, written in Rust. This is not currently endorsed by the LMMS Project.

## Roadmap
* Core
  * Audio engine
    * I'll be honest, I'm not entirely sure what needs to be done here. I'll feel my way through it and ask some devs of 1.3.
  * Project files
    * [ ] Reading
      * [ ] XML format
      * [ ] XML MMPZ (Qt Zlib) format
      * [ ] Non-XML format? (JSON is a valid idea)
    * [ ] Writing
      * [ ] XML format
      * [ ] XML MMPZ (Qt Zlib) format
      * [ ] Non-XML format? (see above)
* Frontends
  * [ ] Exporter
    * Will use core to render a project file headless
  * [ ] GUI
    * Long-term goal, not a priority to focus on getting a stable/maintainable core

We have a [wiki](https://github.com/rdrpenguin04/rmms/wiki/) for documenting our ideas.

## License note
The license is currently subject to relaxing. With permission from the LMMS project, one of the developers (at least) would prefer to relax the license to BSDPlusPatent or similar. The entire RMMS development team must be in consensus to relax the license.
