CREATE TABLE renders(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    created_at timestamptz NOT NULL,
    email text NOT NULL,
    fov_x real NOT NULL,
    fov_y real NOT NULL,
    image_dimension_x integer NOT NULL,
    image_dimension_y integer NOT NULL,
    fundamental_plane_basis_vector_1 real[3] NOT NULL, -- Also serves as the primary direction (longitude 0)
    fundamental_plane_basis_vector_2 real[3] NOT NULL,
    observer_position real[3] NOT NULL,
    latitude real NOT NULL,
    longitude real NOT NULL,
    narrowband_filters real[] NOT NULL,
    broadband_filters text[] NOT NULL,
    image_url text
);
