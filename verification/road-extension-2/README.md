# Remaining city connections

Six ordinary 7.5 m roads continue the reference street plan:

- Midtown avenue: west bank to eastern city street, including a river bridge.
- Market avenue: central shopping blocks between the existing avenues.
- Central cross street: Midtown avenue to the southern perimeter, south of the bank.
- South district street: southern city avenue to the perimeter.
- West bank south link: western street to the southern perimeter across the lake.
- Northwest distributor: northern bridge link to the western city avenue.

Routes preserve existing gameplay buildings and doors. Market avenue begins at
x=0 to avoid the western residences. New endpoints snap to existing samples;
routing tests check connectivity to the city. Initial vehicle spawn locations
remain on established roads; all new streets are available to traffic routes.
No robbery mechanics or save formats changed. Windows sources are synchronized.

Validation logs cover fmt, clippy, workspace tests, release build, road grading
and an OpenGL 2.1 software-rendered map smoke test. Road audit checks 2,305 land
centerline samples and finds no height differences exceeding 20 cm. Existing
road/world tests also check clear carriageways and rail clearance. Map screenshot
comes from the running game. Hardware Windows/Android tests were not performed;
existing ZIP/APK downloads are unchanged by this source update.
