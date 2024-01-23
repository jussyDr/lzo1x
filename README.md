 # LZO1X

Safe Rust port of the LZO1X compression algorithm.

 ## Performance

 ### Decompression

 | source | lzo1x     | lzo-sys   |
 | ------ | --------- | --------- | 
 | bib    | 237,703   | 351,287   |  
 | book1  | 2,153,490 | 3,169,060 |   
 | book2  | 1,509,000 | 2,398,860 |  
 | geo    | 9,404     | 128,597   | 
 | news   | 830,650   | 1,374,860 |   
 | obj1   | 19,630    | 21,425    | 
 | obj2   | 444,815   | 800,790   |
 | paper1 | 120,117   | 135,253   |
 | paper2 | 207,371   | 261,725   |
 | pic    | 400,150   | 1,219,780 |
 | progc  | 76,770    | 85,241    |
 | progl  | 123,778   | 173,122   |
 | progp  | 73,243    | 85,152    |
 | trans  | 134,408   | 223,275   |
