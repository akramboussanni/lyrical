# lyrical
rust cli app that plays a song and shows its lyrics. purely for show

## cmdline args
```
-q/--query [keywords]
searches for keywords in title, album and artist. ex: "2hollis poster boy" 
conflicts with the --title arg 

-t/--title [title] (conflicts with -q)
--artist [artist]
--album [album name]
searches for a song w the specified flag (--title is needed if --query is not supplied)
```

## download
there is a windows build in the releases tab