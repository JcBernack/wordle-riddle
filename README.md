# Solver for Matt Parkers Wordle riddle

## References

- [Matt Parkers Video](https://www.youtube.com/watch?v=_-AfhLQfb6w)
- [Solution by Fred Overflow on YouTube](https://www.youtube.com/watch?v=947Ewgue4DM)
- [Solution by Bored Person on GitHub](https://gist.github.com/BoredPerson/c73f5ccc74a989afc34801e6b394de88)

## Solution times for my algorithm

Solution times for `words_alpha.txt` during development:
```
187.500069011s first attempt
 99.903330992s all loops parallel
 23.887663784s just the outermost loop in parallel
 15.566494685s remove redundant set copying in the non-parallelized loops
 10.085339854s replace range with enumerate()

 18.833595413s collect all matching sets (number of hits 538)
 17.719814862s skip by numerical value
 13.953842087s skip by numerical value with binary search
 13.766282412s skip via slice not iterator
  
  7.686388088s force the first word to include at least one of the two rarest characters (skipping disabled)

 10.735260840s custom alphabet sorted by frequency
 19.228372535s custom alphabet reversed (worst case)
```
