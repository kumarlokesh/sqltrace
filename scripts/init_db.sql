-- SQLTrace Development Database Initialization
-- This file is automatically executed when the PostgreSQL container starts

-- Create extensions that might be useful for query analysis
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
CREATE EXTENSION IF NOT EXISTS btree_gin;
CREATE EXTENSION IF NOT EXISTS btree_gist;

-- Create a sample e-commerce schema for demonstration
CREATE SCHEMA IF NOT EXISTS ecommerce;

-- Users table
CREATE TABLE ecommerce.users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

-- Categories table
CREATE TABLE ecommerce.categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    parent_id INTEGER REFERENCES ecommerce.categories(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Products table
CREATE TABLE ecommerce.products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    price DECIMAL(10,2) NOT NULL CHECK (price >= 0),
    category_id INTEGER NOT NULL REFERENCES ecommerce.categories(id),
    stock_quantity INTEGER DEFAULT 0 CHECK (stock_quantity >= 0),
    sku VARCHAR(50) UNIQUE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Orders table
CREATE TABLE ecommerce.orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES ecommerce.users(id),
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'shipped', 'delivered', 'cancelled')),
    total_amount DECIMAL(10,2) NOT NULL CHECK (total_amount >= 0),
    shipping_address TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Order items table
CREATE TABLE ecommerce.order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES ecommerce.orders(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL REFERENCES ecommerce.products(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL CHECK (unit_price >= 0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Reviews table
CREATE TABLE ecommerce.reviews (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES ecommerce.users(id),
    product_id INTEGER NOT NULL REFERENCES ecommerce.products(id),
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    comment TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, product_id)
);

-- Create indexes for better query performance
CREATE INDEX idx_products_category_id ON ecommerce.products(category_id);
CREATE INDEX idx_products_price ON ecommerce.products(price);
CREATE INDEX idx_products_created_at ON ecommerce.products(created_at);
CREATE INDEX idx_orders_user_id ON ecommerce.orders(user_id);
CREATE INDEX idx_orders_status ON ecommerce.orders(status);
CREATE INDEX idx_orders_created_at ON ecommerce.orders(created_at);
CREATE INDEX idx_order_items_order_id ON ecommerce.order_items(order_id);
CREATE INDEX idx_order_items_product_id ON ecommerce.order_items(product_id);
CREATE INDEX idx_reviews_product_id ON ecommerce.reviews(product_id);
CREATE INDEX idx_reviews_rating ON ecommerce.reviews(rating);

-- Create a view for order summaries (useful for complex query examples)
CREATE VIEW ecommerce.order_summaries AS
SELECT 
    o.id as order_id,
    u.username,
    u.email,
    o.status,
    o.total_amount,
    COUNT(oi.id) as item_count,
    o.created_at
FROM ecommerce.orders o
JOIN ecommerce.users u ON o.user_id = u.id
LEFT JOIN ecommerce.order_items oi ON o.id = oi.order_id
GROUP BY o.id, u.username, u.email, o.status, o.total_amount, o.created_at;

-- Grant permissions
GRANT USAGE ON SCHEMA ecommerce TO postgres;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA ecommerce TO postgres;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA ecommerce TO postgres;
