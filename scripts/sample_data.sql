-- Sample data for SQLTrace demonstration
-- This file populates the database with realistic test data

-- Insert categories
INSERT INTO ecommerce.categories (name, description) VALUES 
    ('Electronics', 'Electronic devices and accessories'),
    ('Computers', 'Computers and computer accessories'),
    ('Smartphones', 'Mobile phones and accessories'),
    ('Home & Garden', 'Home improvement and garden items'),
    ('Books', 'Physical and digital books'),
    ('Clothing', 'Apparel and fashion items'),
    ('Sports', 'Sports equipment and accessories');

-- Insert subcategories
INSERT INTO ecommerce.categories (name, description, parent_id) VALUES 
    ('Laptops', 'Portable computers', 2),
    ('Desktop PCs', 'Desktop computer systems', 2),
    ('Mobile Phones', 'Smartphones and basic phones', 3),
    ('Phone Cases', 'Protective cases for phones', 3),
    ('Fiction', 'Fiction books and novels', 5),
    ('Technical', 'Technical and programming books', 5);

-- Insert users
INSERT INTO ecommerce.users (username, email, first_name, last_name) VALUES 
    ('john_doe', 'john.doe@example.com', 'John', 'Doe'),
    ('jane_smith', 'jane.smith@example.com', 'Jane', 'Smith'),
    ('mike_wilson', 'mike.wilson@example.com', 'Mike', 'Wilson'),
    ('sarah_johnson', 'sarah.j@example.com', 'Sarah', 'Johnson'),
    ('david_brown', 'david.brown@example.com', 'David', 'Brown'),
    ('lisa_davis', 'lisa.davis@example.com', 'Lisa', 'Davis'),
    ('chris_taylor', 'chris.taylor@example.com', 'Chris', 'Taylor'),
    ('amy_anderson', 'amy.anderson@example.com', 'Amy', 'Anderson'),
    ('robert_lee', 'robert.lee@example.com', 'Robert', 'Lee'),
    ('michelle_clark', 'michelle.clark@example.com', 'Michelle', 'Clark');

-- Insert products
INSERT INTO ecommerce.products (name, description, price, category_id, stock_quantity, sku) VALUES 
    ('MacBook Pro 16"', 'Apple MacBook Pro with M2 chip', 2499.00, 8, 25, 'MBP-16-M2-001'),
    ('Dell XPS 13', 'Ultra-portable laptop with Intel i7', 1299.00, 8, 40, 'DELL-XPS13-002'),
    ('iPhone 14 Pro', 'Latest iPhone with Pro camera system', 999.00, 10, 60, 'IPH-14PRO-003'),
    ('Samsung Galaxy S23', 'Android flagship smartphone', 799.00, 10, 45, 'SGS23-004'),
    ('iPad Air', 'Apple tablet with M1 chip', 599.00, 1, 30, 'IPAD-AIR-005'),
    ('Mechanical Keyboard', 'RGB mechanical gaming keyboard', 149.00, 2, 100, 'MECH-KB-006'),
    ('Wireless Mouse', 'Ergonomic wireless mouse', 79.00, 2, 150, 'WIRE-MS-007'),
    ('iPhone Case', 'Protective case for iPhone 14', 29.00, 11, 200, 'IPH14-CASE-008'),
    ('USB-C Hub', '7-in-1 USB-C hub with HDMI', 59.00, 2, 80, 'USBC-HUB-009'),
    ('PostgreSQL Book', 'Complete guide to PostgreSQL', 45.00, 13, 50, 'PSQL-BOOK-010');

-- Insert orders with various dates for interesting time-based queries
INSERT INTO ecommerce.orders (user_id, status, total_amount, shipping_address, created_at) VALUES 
    (1, 'delivered', 2499.00, '123 Main St, New York, NY 10001', '2024-01-15 10:30:00+00'),
    (2, 'delivered', 1378.00, '456 Oak Ave, Los Angeles, CA 90210', '2024-01-20 14:45:00+00'),
    (3, 'shipped', 999.00, '789 Pine Rd, Chicago, IL 60601', '2024-02-01 09:15:00+00'),
    (4, 'confirmed', 878.00, '321 Elm St, Houston, TX 77001', '2024-02-05 16:20:00+00'),
    (1, 'pending', 149.00, '123 Main St, New York, NY 10001', '2024-02-10 11:00:00+00'),
    (5, 'delivered', 638.00, '654 Maple Dr, Phoenix, AZ 85001', '2024-02-12 13:30:00+00'),
    (6, 'delivered', 108.00, '987 Cedar Ln, Philadelphia, PA 19101', '2024-02-15 08:45:00+00'),
    (7, 'cancelled', 2499.00, '147 Birch Ave, San Antonio, TX 78201', '2024-02-18 15:10:00+00'),
    (8, 'shipped', 1358.00, '258 Spruce St, San Diego, CA 92101', '2024-02-20 12:25:00+00'),
    (9, 'delivered', 45.00, '369 Walnut Rd, Dallas, TX 75201', '2024-02-22 10:15:00+00');

-- Insert order items
INSERT INTO ecommerce.order_items (order_id, product_id, quantity, unit_price) VALUES 
    -- Order 1: MacBook Pro
    (1, 1, 1, 2499.00),
    
    -- Order 2: Dell XPS + Accessories
    (2, 2, 1, 1299.00),
    (2, 6, 1, 79.00),
    
    -- Order 3: iPhone 14 Pro
    (3, 3, 1, 999.00),
    
    -- Order 4: Samsung Galaxy + Case
    (4, 4, 1, 799.00),
    (4, 8, 1, 29.00),
    (4, 9, 1, 50.00),
    
    -- Order 5: Mechanical Keyboard
    (5, 6, 1, 149.00),
    
    -- Order 6: iPad Air + Accessories
    (6, 5, 1, 599.00),
    (6, 8, 1, 29.00),
    (6, 9, 1, 10.00),
    
    -- Order 7: Wireless Mouse + USB Hub
    (7, 7, 1, 79.00),
    (7, 9, 1, 29.00),
    
    -- Order 8: MacBook Pro (cancelled)
    (8, 1, 1, 2499.00),
    
    -- Order 9: Dell XPS + Keyboard + Mouse
    (9, 2, 1, 1299.00),
    (9, 6, 1, 149.00),
    (9, 7, 1, 79.00),
    
    -- Order 10: PostgreSQL Book
    (10, 10, 1, 45.00);

-- Insert reviews
INSERT INTO ecommerce.reviews (user_id, product_id, rating, comment) VALUES 
    (1, 1, 5, 'Excellent laptop, very fast and great display quality.'),
    (2, 2, 4, 'Good laptop but battery life could be better.'),
    (2, 7, 5, 'Perfect wireless mouse, very responsive.'),
    (3, 3, 5, 'Amazing camera quality and performance.'),
    (4, 4, 4, 'Great Android phone, good value for money.'),
    (4, 8, 3, 'Case is okay but feels a bit cheap.'),
    (5, 5, 5, 'iPad Air is perfect for work and entertainment.'),
    (6, 7, 4, 'Good mouse but could be more ergonomic.'),
    (6, 9, 5, 'Essential USB-C hub, works perfectly with MacBook.'),
    (9, 10, 5, 'Comprehensive PostgreSQL guide, highly recommended!'),
    (1, 6, 5, 'Best mechanical keyboard I have ever used.'),
    (8, 2, 4, 'Solid laptop for development work.');

-- Update timestamps to create some variety
UPDATE ecommerce.orders SET updated_at = created_at + INTERVAL '2 days' WHERE status IN ('delivered', 'shipped');
UPDATE ecommerce.products SET updated_at = created_at + INTERVAL '1 day' WHERE id % 2 = 0;

-- Add some statistical data by inserting more orders for realistic load testing
DO $$
DECLARE
    i INTEGER;
    user_count INTEGER;
    product_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO user_count FROM ecommerce.users;
    SELECT COUNT(*) INTO product_count FROM ecommerce.products;
    
    -- Create additional orders for performance testing
    FOR i IN 1..50 LOOP
        INSERT INTO ecommerce.orders (user_id, status, total_amount, shipping_address, created_at) 
        SELECT 
            (random() * (user_count - 1) + 1)::INTEGER,
            CASE (random() * 4)::INTEGER 
                WHEN 0 THEN 'pending'
                WHEN 1 THEN 'confirmed' 
                WHEN 2 THEN 'shipped'
                WHEN 3 THEN 'delivered'
                ELSE 'cancelled'
            END,
            (random() * 2000 + 50)::DECIMAL(10,2),
            'Generated Address ' || i::TEXT,
            CURRENT_TIMESTAMP - (random() * INTERVAL '90 days');
    END LOOP;
END $$;
