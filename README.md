 # LZO1X

Safe Rust port of the LZO1X compression algorithm.

 ## Performance

 ### Decompression

 | source | lzo1x     | lzo-sys   |
 | ------ | --------- | --------- | 
 | bib    | 325,483   | 353,355   |  
 | book1  | 2,831,630 | 3,184,930 |   
 | book2  | 1,976,020 | 2,385,990 |  
 | geo    | 9,264     | 125,459   | 
 | news   | 1,099,210 | 1,375,110 |   
 | obj1   | 35,076    | 20,427    | 
 | obj2   | 579,470   | 798,335   |
 | paper1 | 163,318   | 130,845   |
 | paper2 | 276,866   | 263,043   |
 | pic    | 452,370   | 1,219,140 |
 | progc  | 105,577   | 78,730    |
 | progl  | 161,954   | 178,545   |
 | progp  | 94,940    | 83,282    |
 | trans  | 173,874   | 221,531   |
