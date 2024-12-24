#!/usr/bin/perl


my $total = $#WARGY + 1;
my $counter = 1;
my $scriptname = $0;

print "Total Args Passed To $scriptname : $total\n";

foreach my $a(@ARGV) {
    print "Arg $counter : $a\n";
    $counter++;
}