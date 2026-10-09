# Segment

A segment is an individually flashed unit: a header at the start of a flash sector, wrapping the data that follows it. An image is a full snapshot of flash, made up of segments. VGBL defines a shared segment header structure to be used or expanded upon by HAL implementations
