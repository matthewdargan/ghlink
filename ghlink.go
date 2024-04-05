// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Ghlink creates GitHub permanent links to specified file lines of files
// hosted in a GitHub repository.
//
// Usage:
//
//	ghlink file [line1] [line2]
//
// “ghlink file” prints a link to file.
//
// “ghlink file line1” prints a link to line1 in file.
//
// “ghlink file line1 line2” prints a link to lines line1 through line2 in file.
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
//	$ ghlink README.md 3
//
// Print a link to lines 3 through 8 in README.md:
//
//	$ ghlink README.md 3 8
package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

func usage() {
	fmt.Fprintf(os.Stderr, "usage: ghlink file [line1] [line2]\n")
	os.Exit(2)
}

func main() {
	log.SetPrefix("ghlink: ")
	log.SetFlags(0)
	flag.Usage = usage
	flag.Parse()
	if flag.NArg() < 1 || flag.NArg() > 3 {
		usage()
	}
	args := flag.Args()
	f := args[0]
	var l1, l2 string
	if flag.NArg() > 1 {
		l1 = args[1]
	}
	if flag.NArg() > 2 {
		l2 = args[2]
	}
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
	if l1 != "" {
		n, err := strconv.Atoi(l1)
		if err != nil {
			log.Fatalf("invalid line %q", l1)
		}
		url += fmt.Sprintf("#L%d", n)
	}
	if l2 != "" {
		n, err := strconv.Atoi(l2)
		if err != nil {
			log.Fatalf("invalid line %q", l2)
		}
		url += fmt.Sprintf("-L%d", n)
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
