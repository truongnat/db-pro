-- Expands lab.seed_manifest (the JSON mapping) into customers, products, orders, items, and events.
DO $$
DECLARE
    doc jsonb := (SELECT document FROM lab.seed_manifest WHERE id = 1);
    customer_n int := (doc #>> '{volumes,customers}')::int;
    product_n int := (doc #>> '{volumes,products}')::int;
    order_n int := (doc #>> '{volumes,orders}')::int;
    event_n int := (doc #>> '{volumes,events}')::int;
    name_n int := jsonb_array_length(doc->'given_names');
    family_n int := jsonb_array_length(doc->'family_names');
    city_n int := jsonb_array_length(doc->'cities');
    category_n int := jsonb_array_length(doc->'categories');
    status_n int := jsonb_array_length(doc->'order_statuses');
    event_type_n int := jsonb_array_length(doc->'event_types');
BEGIN
    INSERT INTO lab.categories (slug, name)
    SELECT item->>'slug', item->>'name'
    FROM jsonb_array_elements(doc->'categories') AS item;

    INSERT INTO lab.products (sku, name, category_id, price_cents, attributes)
    SELECT
        'SKU-' || lpad(i::text, 6, '0'),
        (doc->'categories'->((i - 1) % category_n)->>'name') || ' ' || i,
        ((i - 1) % category_n) + 1,
        500 + ((i * 37) % 250000),
        jsonb_build_object(
            'category', doc->'categories'->((i - 1) % category_n)->>'slug',
            'color', (ARRAY['black','silver','oak','navy'])[1 + ((i - 1) % 4)],
            'weight_g', 80 + (i % 4000),
            'mapped_from', 'seed-map.json'
        )
    FROM generate_series(1, product_n) AS i;

    INSERT INTO lab.customers (email, full_name, city, profile)
    SELECT
        'customer' || i || '@lab.dbpro.test',
        (doc->'given_names'->>((i - 1) % name_n)) || ' ' || (doc->'family_names'->>((i - 1) % family_n)),
        doc->'cities'->>((i - 1) % city_n),
        jsonb_build_object(
            'city', doc->'cities'->>((i - 1) % city_n),
            'segment', (ARRAY['new','active','vip'])[1 + ((i - 1) % 3)],
            'locale', 'vi-VN',
            'mapped_from', 'seed-map.json'
        )
    FROM generate_series(1, customer_n) AS i;

    INSERT INTO lab.orders (customer_id, status, total_cents, placed_at, meta)
    SELECT
        ((i - 1) % customer_n) + 1,
        doc->'order_statuses'->>((i - 1) % status_n),
        0,
        timestamptz '2024-01-01' + ((i % 600) || ' days')::interval + ((i % 86400) || ' seconds')::interval,
        jsonb_build_object(
            'channel', (ARRAY['web','store','agent'])[1 + ((i - 1) % 3)],
            'note', 'expanded from seed-map volumes.orders',
            'mapped_from', 'seed-map.json'
        )
    FROM generate_series(1, order_n) AS i;

    INSERT INTO lab.order_items (order_id, product_id, quantity, unit_price_cents)
    SELECT
        o.id,
        ((o.id + item_n - 1) % product_n) + 1,
        1 + ((o.id + item_n) % 4),
        500
    FROM lab.orders o
    CROSS JOIN generate_series(1, 2) AS item_n;

    UPDATE lab.order_items oi
    SET unit_price_cents = p.price_cents
    FROM lab.products p
    WHERE p.id = oi.product_id;

    UPDATE lab.orders o
    SET total_cents = sums.total
    FROM (
        SELECT order_id, sum(quantity * unit_price_cents)::bigint AS total
        FROM lab.order_items
        GROUP BY order_id
    ) AS sums
    WHERE sums.order_id = o.id;

    INSERT INTO lab.events (event_type, payload, occurred_at)
    SELECT
        doc->'event_types'->>((i - 1) % event_type_n),
        jsonb_build_object(
            'actor', 'customer' || (((i - 1) % customer_n) + 1),
            'table', (ARRAY['lab.orders','lab.products','lab.customers','lab.events'])[1 + ((i - 1) % 4)],
            'rows', 1 + (i % 200),
            'ok', (i % 17) <> 0,
            'mapped_from', 'seed-map.json'
        ),
        timestamptz '2025-06-01' + ((i % 120) || ' days')::interval
    FROM generate_series(1, event_n) AS i;
END $$;
