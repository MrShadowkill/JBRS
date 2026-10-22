# Road extensions

Five ordinary 7.5 m streets follow the reference map's western bank, northern
river connection, southern perimeter, southern city connection and eastern loop.
The northern bridge avoids the existing police buildings; the southern city
connection passes east of the gym. Existing gameplay buildings remain in place.
Endpoints snap to existing road samples for traffic graph connectivity.
Initial fleet spawns stay on established arteries to preserve traffic spacing;
new streets participate in routes and patrol destinations. Street lamp colliders
are excluded from every intersecting carriageway, not only their own street.

The southern perimeter crosses both the main river and western lake. A smooth
lake approach raises the deck above water; world road support queries provide the
same collision surface as the visible ribbon. Existing bridge mesh supports and
railings apply automatically. Terrain stays below bridges rather than filling
waterways. No robbery mechanics changed.

Validation: workspace fmt, clippy, tests and release build logs are stored here.
Road audit: 2,002 land centerline samples, zero height differences above 20 cm.
Road/world tests cover unobstructed lanes, train clearance, bridge support and
routing connectivity for every new street endpoint. Screenshot is the actual
OpenGL 2.1 map view, not a mockup. Native Windows and Android hardware not tested
for this source update. Existing downloadable 0.7.6 archives predate this change.
