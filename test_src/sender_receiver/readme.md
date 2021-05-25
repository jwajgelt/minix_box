sender (endpoint 41): sends a message with the first byte equal to 42 to receiver

receiver: receives a message from sender

sendrec_39: receives message from 40, changes the second byte, and sends it back

sendrec_40: sendreceives message to/from 39


All files are compiled on Minix 3.4 with flags ` -g -static -nostartfiles `