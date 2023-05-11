CREATE TABLE renders(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    image_url text,
    created_at timestamp NOT NULL,
    fov_x real NOT NULL,
    fov_y real NOT NULL,
    image_dimension_x integer NOT NULL,
    image_dimension_y integer NOT NULL,
    fundamental_plane_bases real[2][3] NOT NULL,
    primary_direction real[3] NOT NULL,
    observer_position real[3] NOT NULL,
    latitude real NOT NULL,
    longitude real NOT NULL,
    broadband_filters text[],
    narrowband_filters real[]
);