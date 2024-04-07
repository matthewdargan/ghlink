// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Ghlink creates GitHub permanent links to specified file lines of files
// hosted in a GitHub repository.
//
// Usage:
//
//	ghlink [-l1 line1 [-l2 line2] | -s text] file
//
// “ghlink file” prints a link to file.
//
// “ghlink -l1 line1 file” prints a link to line1 in file.
//
// “ghlink -l1 line1 -l2 line2 file” prints a link to lines line1 through
// line2 in file.
//
// “ghlink -s text file” prints a link to lines matching text in file. If
// text is ‘-’, the standard input is used.
//
// The “git” program must be on the system's PATH environment variable.
//
// Examples:
//
// Print a link to README.md:
//
//	$ ghlink README.md
//
// Print a link to line 3 in README.md:
//
//	$ ghlink -l1 3 README.md
//
// Print a link to lines 3 through 8 in README.md:
//
//	$ ghlink -l1 3 -l2 8 README.md
//
// Print a link to lines matching "Usage:\n\n    ghlink file":
//
//	$ ghlink -s 'Usage:\n\n    ghlink file' README.md
package main

import (
	"bufio"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

var (
	line1  = flag.Int("l1", -1, "print link to start line number")
	line2  = flag.Int("l2", -1, "print link to end line number")
	search = flag.String("s", "", "print link to matching text")
)

func usage() {
	fmt.Fprintf(os.Stderr, "usage: ghlink [-l1 line1 [-l2 line2] | -s text] file\n")
	os.Exit(2)
}

func main() {
	log.SetPrefix("ghlink: ")
	log.SetFlags(0)
	flag.Usage = usage
	flag.Parse()
	if flag.NArg() != 1 {
		usage()
	}
	if *line1 == -1 && *line2 != -1 {
		usage()
	}
	if *search != "" && (*line1 != -1 || *line2 != -1) {
		usage()
	}
	f := flag.Arg(0)
	r, err := repo(f)
	if err != nil {
		log.Fatalf("cannot get repo: %v", err)
	}
	c, err := commit(f)
	if err != nil {
		log.Fatalf("cannot get commit: %v", err)
	}
	p, err := relPath(f)
	if err != nil {
		log.Fatalf("cannot get relative path: %v", err)
	}
	url := fmt.Sprintf("https://github.com/%s/blob/%s/%s", r, c, p)
	if *line1 != -1 {
		url += fmt.Sprintf("#L%d", *line1)
	}
	if *line2 != -1 {
		url += fmt.Sprintf("-L%d", *line2)
	}
	if *search == "-" {
		s, err := io.ReadAll(os.Stdin)
		if err != nil {
			log.Fatal(err)
		}
		*search = string(s)
	}
	if *search != "" {
		l, err := searchLines(f, *search)
		if err != nil {
			log.Fatalf("cannot search lines: %v", err)
		}
		url += fmt.Sprintf("#L%d", l[0])
		if len(l) > 1 {
			url += fmt.Sprintf("-L%d", l[len(l)-1])
		}
	}
	fmt.Println(url)
}

const remotePrefix = "git@github.com:"

func repo(f string) (string, error) {
	cmd := exec.Command("git", "remote", "get-url", "origin")
	cmd.Dir = filepath.Dir(f)
	cmd.Stderr = os.Stderr
	o, err := cmd.Output()
	if err != nil {
		return "", err
	}
	repoURL := strings.TrimSpace(string(o))
	if !strings.HasPrefix(repoURL, remotePrefix) {
		return "", fmt.Errorf("unexpected prefix for remote %q (want %q)", repoURL, remotePrefix)
	}
	return strings.TrimSuffix(strings.TrimPrefix(repoURL, remotePrefix), ".git"), nil
}

func commit(f string) (string, error) {
	cmd := exec.Command("git", "rev-parse", "HEAD")
	cmd.Dir = filepath.Dir(f)
	cmd.Stderr = os.Stderr
	o, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(string(o)), nil
}

func relPath(f string) (string, error) {
	cmd := exec.Command("git", "rev-parse", "--show-prefix")
	cmd.Dir = filepath.Dir(f)
	cmd.Stderr = os.Stderr
	o, err := cmd.Output()
	if err != nil {
		return "", err
	}
	return filepath.Join(strings.TrimSpace(string(o)), filepath.Base(f)), nil
}

func searchLines(f, t string) ([]int, error) {
	if t[len(t)-1] == '\n' {
		t = t[:len(t)-1]
	}
	searchLines := strings.Split(t, "\n")
	file, err := os.Open(f)
	if err != nil {
		return nil, err
	}
	defer file.Close()
	s := bufio.NewScanner(file)
	var lines []int
	for i, l := 0, 1; s.Scan() && i < len(searchLines); l++ {
		if strings.Contains(s.Text(), searchLines[i]) {
			lines = append(lines, l)
			i++
		}
	}
	if err := s.Err(); err != nil {
		return nil, err
	}
	if len(lines) == 0 {
		return nil, fmt.Errorf("file %q does not contain string %q", f, t)
	}
	return lines, nil
}
