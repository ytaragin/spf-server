#!/bin/zsh

# pdftotext -raw  -nopgbrk

foreach f (**/*.pdf) 
    echo $f
    pdftotext -raw  -nopgbrk $f
end