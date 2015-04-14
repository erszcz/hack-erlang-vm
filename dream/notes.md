# 2015-04-14 Code loading

How?

1.  beam-atoms <- atom table from beam file chunk
2.  for each exported function:
    - get name atom index
    - lookup in beam-atoms to find the atom itself
    - store/get-index from emulator's atom table
    - put into export table
