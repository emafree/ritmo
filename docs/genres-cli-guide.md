# Genre CLI Commands and Filtering Functionality

This guide provides comprehensive documentation on the genre command-line interface (CLI), including the various commands available and filtering options for managing content efficiently.

## Table of Contents
1. [Introduction](#introduction)
2. [Available Commands](#available-commands)
   - [Add Genre](#add-genre)
   - [Remove Genre](#remove-genre)
   - [List Genres](#list-genres)
3. [Filtering Functionality](#filtering-functionality)
   - [By Name](#by-name)
   - [By Characteristics](#by-characteristics)
4. [Examples](#examples)
5. [Conclusion](#conclusion)

## Introduction
The Genre CLI allows users to manage genres effectively, providing functionality to add, remove, and list genres.

## Available Commands

### Add Genre
**Command:** `genre add <name>`  
**Description:** Adds a new genre with the given name.  
**Usage:**  
```bash
genre add "New Genre"
```  

### Remove Genre
**Command:** `genre remove <name>`  
**Description:** Removes the genre specified by the name.  
**Usage:**  
```bash
genre remove "Old Genre"
```  

### List Genres
**Command:** `genre list`  
**Description:** Lists all current genres.  
**Usage:**  
```bash
genre list
```

## Filtering Functionality

The filtering functionality allows users to refine their results when listing genres.

### By Name
**Command:** `genre list --filter <name>`  
**Description:** Filters genres by name.  
**Usage:**  
```bash
genre list --filter "Horror"
```

### By Characteristics
**Command:** `genre list --filter <characteristic>`  
**Description:** Filters genres by specific characteristics.  
**Usage:**  
```bash
genre list --filter "Dark"
```

## Examples
- Adding a new genre:  
```bash
genre add "Sci-Fi"
```
- Removing a genre:  
```bash
genre remove "Romance"
```
- Listing all genres:  
```bash
genre list
```

## Conclusion
This guide outlines the essential Genre CLI commands along with filtering functionalities. Utilize these commands to manage your genres effectively and efficiently.