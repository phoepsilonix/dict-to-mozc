#!/bin/bash

cut -f2 -d" " id.def|awk '{FS=","}{print "mapping.add_mapping(""\""$1"\""", \""$1","$2","$3","$4","$5","$6"\");"}'|sort -u
