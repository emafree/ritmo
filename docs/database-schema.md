# Database Schema Documentation

## Tables

### Users
- **user_id**: INT, Primary Key
- **username**: VARCHAR(255), Unique
- **email**: VARCHAR(255), Unique
- **created_at**: TIMESTAMP
- **updated_at**: TIMESTAMP

### Genres
- **genre_id**: INT, Primary Key
- **name**: VARCHAR(255), Unique
- **description**: TEXT

### Songs
- **song_id**: INT, Primary Key
- **title**: VARCHAR(255)
- **artist_id**: INT, Foreign Key (references Artists)
- **genre_id**: INT, Foreign Key (references Genres)
- **album_id**: INT, Foreign Key (references Albums)
- **duration**: TIME

### Artists
- **artist_id**: INT, Primary Key
- **name**: VARCHAR(255), Unique
- **bio**: TEXT

### Albums
- **album_id**: INT, Primary Key
- **title**: VARCHAR(255)
- **release_date**: DATE
- **artist_id**: INT, Foreign Key (references Artists)

### Relationships
- **Users** can create multiple **Songs**.
- Each **Song** belongs to one or more **Genres**.
- Each **Artist** can release multiple **Albums**.
- Each **Album** can contain multiple **Songs**.

## Genres Entity
The Genres table is essential for categorizing songs into types, providing a way to organize tracks, and enabling users to filter and search by genre.
- **genre_id** serves as the unique identifier for each genre.
- **name** provides a human-readable identifier for the genre.
- **description** offers a detailed explanation of the characteristics associated with the genre.
