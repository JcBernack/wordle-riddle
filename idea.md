# Ideas

## Generalizations

Parameters
- WORD_LENGTH, default `5` 
- WORD_COUNT, default `5`
- alphabet, default is standard latin 26 characters `ABCDEFGHIJKLMNOPQRSTUVWXY`

Improvements
- get WORD_LENGTH and WORD_COUNT from CLI arguments
- get input filename from CLI argument
- determine alphabet from the input file (find all characters appearing in the file)
- calculate amount of unused characters: alphabet.len() - WORD_LENGTH * WORD_COUNT
  - if less than 0: abort, no solution possible
  - if 0: every character must be used
  - otherwise N characters will not be used (build first_words using the N+1 rarest letters)
- only iterate over words that contain a missing character

## Different approach

- the more words are in the set, the more bits are set
- the more bits are set the higher the chances of "duplicate search paths"
- try to merge these to reduce work
- this is similar to the removal of anagrams as one of the first steps
- anagrams have identical search paths in this problem
- the approach is basically to find and merge "set anagrams" without losing all the information about which words were used to build them

mapping of binary representation to a structure holding words
sort and merge such that the binaries are unique afterwards and
all the words of duplicate binaries are merged into one

```
before
ABCDE--------------------- abcde
ABCDE--------------------- abced (anagram)
ABC--FG------------------- abcfg
---DE--HIJ---------------- dehij
-----FGHIJ---------------- fghij

after
ABCDE--------------------- (abcde, abced)
ABC--FG------------------- (abcfg)
---DE--HIJ---------------- (dehij)
-----FGHIJ---------------- (fghij)
```

then build all combinations of entries which have no overlap: (a & b == 0)
while also merging their result structure
```
ABCDEFGHIJ---------------- ((abcde, abced), (fghij)) combination of 1 and 4
ABCDEFGHIJ---------------- ((abcfg), (dehij)) combination of 2 and 3
```
then sort and merge by binary representation again
```
ABCDEFGHIJ---------------- (((abcde, abced), (fghij)), ((abcfg), (dehij)))
```

repeat 5 times
