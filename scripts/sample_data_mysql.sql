-- Sample data for MySQL SQLTrace testing
USE sqltrace_dev;

-- Insert categories
INSERT INTO categories (name, description, parent_id) VALUES
('Electronics', 'Electronic devices and accessories', NULL),
('Computers', 'Computer hardware and software', 1),
('Mobile Devices', 'Smartphones and tablets', 1),
('Books', 'Physical and digital books', NULL),
('Programming', 'Programming and technical books', 4),
('Fiction', 'Fiction literature', 4),
('Clothing', 'Apparel and accessories', NULL),
('Home & Garden', 'Home improvement and gardening', NULL);

-- Insert suppliers
INSERT INTO suppliers (name, contact_email, contact_phone, address, city, country) VALUES
('TechCorp Ltd', 'sales@techcorp.com', '+1-555-0101', '123 Tech Street', 'San Francisco', 'USA'),
('Global Electronics', 'info@globalelec.com', '+1-555-0102', '456 Circuit Ave', 'Austin', 'USA'),
('BookWorld Distribution', 'orders@bookworld.com', '+1-555-0103', '789 Library Lane', 'New York', 'USA'),
('Fashion Forward Inc', 'wholesale@fashionforward.com', '+1-555-0104', '321 Style Boulevard', 'Los Angeles', 'USA'),
('Home Solutions Co', 'contact@homesolutions.com', '+1-555-0105', '654 Garden Way', 'Seattle', 'USA');

-- Insert products
INSERT INTO products (name, description, price, category_id, supplier_id, stock_quantity, weight) VALUES
('MacBook Pro 16"', 'High-performance laptop for professionals', 2499.99, 2, 1, 50, 2.0),
('iPhone 14 Pro', 'Latest smartphone with advanced camera', 999.99, 3, 1, 150, 0.2),
('Dell XPS 13', 'Ultrabook for business and personal use', 1299.99, 2, 2, 75, 1.2),
('iPad Air', 'Versatile tablet for work and entertainment', 599.99, 3, 1, 100, 0.46),
('Clean Code', 'A handbook of agile software craftsmanship', 29.99, 5, 3, 200, 0.7),
('The Great Gatsby', 'Classic American literature', 12.99, 6, 3, 300, 0.3),
('JavaScript: The Good Parts', 'Essential JavaScript programming guide', 24.99, 5, 3, 150, 0.4),
('Casual T-Shirt', 'Comfortable cotton t-shirt', 19.99, 7, 4, 500, 0.2),
('Jeans', 'Classic blue denim jeans', 59.99, 7, 4, 200, 0.6),
('Garden Hose', '50ft expandable garden hose', 39.99, 8, 5, 100, 2.5),
('Wireless Mouse', 'Ergonomic wireless mouse', 25.99, 2, 2, 300, 0.1),
('Bluetooth Headphones', 'Noise-cancelling wireless headphones', 149.99, 1, 2, 80, 0.3);

-- Insert users with varied data for interesting queries
INSERT INTO users (username, email, first_name, last_name, age, department, salary, is_active) VALUES
('john_doe', 'john.doe@example.com', 'John', 'Doe', 28, 'Engineering', 75000.00, TRUE),
('jane_smith', 'jane.smith@example.com', 'Jane', 'Smith', 32, 'Marketing', 65000.00, TRUE),
('bob_wilson', 'bob.wilson@example.com', 'Bob', 'Wilson', 45, 'Engineering', 95000.00, TRUE),
('alice_brown', 'alice.brown@example.com', 'Alice', 'Brown', 29, 'Sales', 55000.00, TRUE),
('charlie_davis', 'charlie.davis@example.com', 'Charlie', 'Davis', 35, 'Engineering', 85000.00, TRUE),
('diana_miller', 'diana.miller@example.com', 'Diana', 'Miller', 27, 'Marketing', 60000.00, TRUE),
('eve_garcia', 'eve.garcia@example.com', 'Eve', 'Garcia', 31, 'Sales', 58000.00, TRUE),
('frank_martinez', 'frank.martinez@example.com', 'Frank', 'Martinez', 42, 'Engineering', 92000.00, TRUE),
('grace_anderson', 'grace.anderson@example.com', 'Grace', 'Anderson', 26, 'Marketing', 62000.00, TRUE),
('henry_taylor', 'henry.taylor@example.com', 'Henry', 'Taylor', 38, 'Sales', 68000.00, TRUE),
('inactive_user', 'inactive@example.com', 'Inactive', 'User', 30, 'Support', 45000.00, FALSE);

-- Insert orders with realistic patterns
INSERT INTO orders (user_id, total_amount, status, shipping_address, billing_address) VALUES
(1, 2499.99, 'delivered', '123 Main St, City, State 12345', '123 Main St, City, State 12345'),
(1, 149.99, 'delivered', '123 Main St, City, State 12345', '123 Main St, City, State 12345'),
(2, 999.99, 'shipped', '456 Oak Ave, Town, State 67890', '456 Oak Ave, Town, State 67890'),
(2, 42.98, 'delivered', '456 Oak Ave, Town, State 67890', '456 Oak Ave, Town, State 67890'),
(3, 1299.99, 'processing', '789 Pine Rd, Village, State 11111', '789 Pine Rd, Village, State 11111'),
(4, 79.98, 'delivered', '321 Elm St, Hamlet, State 22222', '321 Elm St, Hamlet, State 22222'),
(5, 1799.98, 'delivered', '654 Maple Dr, Borough, State 33333', '654 Maple Dr, Borough, State 33333'),
(6, 29.99, 'delivered', '987 Cedar Ln, Township, State 44444', '987 Cedar Ln, Township, State 44444'),
(7, 159.98, 'cancelled', '147 Birch Way, District, State 55555', '147 Birch Way, District, State 55555'),
(8, 2599.98, 'delivered', '258 Spruce Ct, County, State 66666', '258 Spruce Ct, County, State 66666'),
(9, 599.99, 'pending', '369 Willow Pl, Region, State 77777', '369 Willow Pl, Region, State 77777'),
(10, 89.97, 'delivered', '741 Aspen Blvd, Area, State 88888', '741 Aspen Blvd, Area, State 88888');

-- Insert order items
INSERT INTO order_items (order_id, product_id, quantity, unit_price, total_price) VALUES
-- Order 1: MacBook Pro
(1, 1, 1, 2499.99, 2499.99),
-- Order 2: Bluetooth Headphones
(2, 12, 1, 149.99, 149.99),
-- Order 3: iPhone 14 Pro
(3, 2, 1, 999.99, 999.99),
-- Order 4: Books (Clean Code + The Great Gatsby)
(4, 5, 1, 29.99, 29.99),
(4, 6, 1, 12.99, 12.99),
-- Order 5: Dell XPS 13
(5, 3, 1, 1299.99, 1299.99),
-- Order 6: Clothing (T-Shirt + Jeans)
(6, 8, 1, 19.99, 19.99),
(6, 9, 1, 59.99, 59.99),
-- Order 7: Multiple tech items
(7, 4, 1, 599.99, 599.99),
(7, 1, 1, 2499.99, 2499.99),
(7, 11, 1, 25.99, 25.99),
-- Order 8: Book
(8, 5, 1, 29.99, 29.99),
-- Order 9: Clothing (cancelled)
(9, 8, 2, 19.99, 39.98),
(9, 9, 2, 59.99, 119.98),
-- Order 10: Tech accessories
(10, 1, 1, 2499.99, 2499.99),
(10, 12, 1, 149.99, 149.99),
-- Order 11: iPad Air
(11, 4, 1, 599.99, 599.99),
-- Order 12: Garden and tech items
(12, 10, 1, 39.99, 39.99),
(12, 11, 1, 25.99, 25.99),
(12, 7, 1, 24.99, 24.99);

-- Insert additional orders for more complex query scenarios
INSERT INTO orders (user_id, total_amount, status, created_at) VALUES
(1, 75.98, 'delivered', DATE_SUB(NOW(), INTERVAL 30 DAY)),
(2, 299.97, 'delivered', DATE_SUB(NOW(), INTERVAL 15 DAY)),
(3, 149.99, 'delivered', DATE_SUB(NOW(), INTERVAL 60 DAY)),
(4, 89.97, 'delivered', DATE_SUB(NOW(), INTERVAL 45 DAY)),
(5, 199.98, 'delivered', DATE_SUB(NOW(), INTERVAL 7 DAY));

-- Insert corresponding order items for the additional orders
INSERT INTO order_items (order_id, product_id, quantity, unit_price, total_price) VALUES
(13, 8, 2, 19.99, 39.98),
(13, 10, 1, 39.99, 39.99),
(14, 5, 3, 29.99, 89.97),
(14, 6, 5, 12.99, 64.95),
(14, 7, 6, 24.99, 149.99),
(15, 12, 1, 149.99, 149.99),
(16, 8, 1, 19.99, 19.99),
(16, 9, 1, 59.99, 59.99),
(16, 10, 1, 39.99, 39.98),
(17, 11, 2, 25.99, 51.98),
(17, 7, 6, 24.99, 149.94);
