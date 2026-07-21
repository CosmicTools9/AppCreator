// AliothStudio DSL - 全功能示例模型
// 展示 Phase 20-27 所有 DSL 特性

// ==================== 枚举定义 ====================

enum OrderStatus {
    PENDING,
    CONFIRMED,
    SHIPPED,
    DELIVERED,
    CANCELLED,
    REFUNDED
}

enum Priority {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL
}

enum UserRole {
    ADMIN,
    MANAGER,
    EDITOR,
    VIEWER
}

// ==================== 异常定义 ====================

@abstract
@httpStatus(400)
exception BusinessException {
    message: String
    code: String
    timestamp: DateTime
}

@extends(BusinessException)
@httpStatus(400)
exception ValidationError {
    field: String
    errors: String[]
}

@extends(BusinessException)
@httpStatus(404)
exception NotFoundError {
    resource_type: String
    resource_id: String
}

@extends(BusinessException)
@httpStatus(409)
exception ConflictError {
    conflict_field: String
    conflict_value: String
}

// ==================== 抽象基类 ====================

@abstract
@permission(admin, [create, read, update, delete])
@quality(completeness, 0.95)
entity BaseEntity {
    @quality(uniqueness, 1.0)
    id: UUID
    
    created_at: DateTime
    updated_at: DateTime
    
    @readRole([admin, manager, editor, viewer])
    @writeRole([admin, manager])
    created_by: String
}

@abstract
@extends(BaseEntity)
entity AuditableEntity {
    @readRole([admin, manager])
    @writeRole([admin])
    audit_log: String
    
    version: Integer
}

// ==================== 用户管理 ====================

@extends(BaseEntity)
@statemachine
@states([ACTIVE, INACTIVE, SUSPENDED, DELETED])
@scene(domain: "user_management", context: "authentication")
@position(T: "2024", S: "global", Fa: "user", Fu: "management")
entity User {
    @unique @required
    @constraint("length(email) >= 5", error: "Email too short")
    @throws(ValidationError, "email.isEmpty()")
    @quality(uniqueness, 1.0)
    @readRole([admin, manager])
    email: String
    
    @minLength(2) @maxLength(100)
    @quality(completeness, 0.99)
    first_name: String
    
    @minLength(2) @maxLength(100)
    @quality(completeness, 0.99)
    last_name: String
    
    @writeRole([admin])
    password_hash: String
    
    role: UserRole
    
    @validFrom
    @validTo
    valid_period: DateTimeRange
    
    @onCreate(fn: "hashPassword")
    @onUpdate(fn: "updateTimestamp")
    status: String
    
    relation profile -> Profile
    relation orders -> Order[*]
    relation posts -> Post[*]
    
    @transition(event: "activate", from: INACTIVE, to: ACTIVE)
    @transition(event: "suspend", from: [ACTIVE, INACTIVE], to: SUSPENDED)
    @transition(event: "delete", from: [ACTIVE, INACTIVE, SUSPENDED], to: DELETED)
}

@extends(BaseEntity)
@scene(domain: "user_management")
entity Profile {
    @quality(completeness, 0.80)
    ?bio: Text
    
    ?avatar_url: String
    ?website: String
    
    @min(0) @max(150)
    ?age: Integer
    
    relation user -> User
}

// ==================== 订单管理 ====================

@extends(AuditableEntity)
@statemachine
@states([PENDING, CONFIRMED, PROCESSING, SHIPPED, DELIVERED, CANCELLED])
@scene(domain: "order_management", context: "fulfillment")
@position(T: "2024-Q1", S: "ecommerce", Fa: "transaction", Fu: "processing")
@permission(admin, [create, read, update, delete])
@permission(manager, [read, update])
@permission(customer, [read])
@quality(accuracy, 0.98)
entity Order {
    @unique
    @constraint("orderNumber.matches('ORD-[0-9]{8}')")
    order_number: String
    
    @min(0)
    @quality(accuracy, 0.99)
    subtotal: Decimal
    
    @min(0)
    tax_amount: Decimal
    
    @min(0)
    shipping_amount: Decimal
    
    @min(0)
    total_amount: Decimal
    
    priority: Priority
    
    @onCreate(fn: "generateOrderNumber")
    @onTransition(from: PENDING, to: CONFIRMED)
    fn sendConfirmationEmail() { }
    
    @transition(event: "confirm", from: PENDING, to: CONFIRMED, guard: "itemsInStock")
    @transition(event: "process", from: CONFIRMED, to: PROCESSING)
    @transition(event: "ship", from: PROCESSING, to: SHIPPED)
    @transition(event: "deliver", from: SHIPPED, to: DELIVERED)
    @transition(event: "cancel", from: [PENDING, CONFIRMED], to: CANCELLED)
    status: OrderStatus
    
    relation customer -> User
    relation items -> OrderItem[*]
    relation shipments -> Shipment[*]
    
    @rule(name: "minimumOrder", condition: "total_amount >= 10", error: "Minimum order is $10")
    @rule(name: "stockCheck", condition: "items.all(i => i.inStock)", action: "reserveInventory")
    validation_rules: String
}

@extends(BaseEntity)
entity OrderItem {
    @min(1) @max(999)
    quantity: Integer
    
    @min(0)
    unit_price: Decimal
    
    @min(0)
    discount_amount: Decimal
    
    @min(0)
    total_price: Decimal
    
    relation order -> Order
    relation product -> Product
}

// ==================== 产品管理 ====================

@extends(AuditableEntity)
@scene(domain: "product_management")
@permission(admin, [create, read, update, delete])
@permission(manager, [create, read, update])
@permission(editor, [read, update])
@permission(viewer, [read])
entity Product {
    @unique @required
    @quality(uniqueness, 1.0)
    sku: String
    
    @minLength(1) @maxLength(200)
    @quality(completeness, 0.95)
    name: String
    
    ?description: Text
    
    @min(0)
    @quality(accuracy, 0.98)
    price: Decimal
    
    @min(0)
    cost: Decimal
    
    @quality(consistency, 0.90)
    stock_quantity: Integer
    
    @factor(type: "inventory")
    @factorDimension([quantity, value, location])
    inventory_metrics: String
    
    relation category -> Category
    relation supplier -> Supplier
    relation order_items -> OrderItem[*]
    
    @rule(name: "positivePrice", condition: "price > 0", error: "Price must be positive")
    @rule(name: "stockAlert", condition: "stock_quantity < 10", action: "sendLowStockAlert")
    business_rules: String
}

@extends(BaseEntity)
entity Category {
    @unique
    slug: String
    
    name: String
    
    ?description: Text
    
    @sortOrder
    sort_order: Integer
    
    relation parent -> Category
    relation children -> Category[*]
    relation products -> Product[*]
}

@extends(BaseEntity)
entity Supplier {
    @unique
    code: String
    
    name: String
    
    @email
    email: String
    
    ?phone: String
    ?address: Text
    
    relation products -> Product[*]
}

// ==================== 物流管理 ====================

@extends(BaseEntity)
@statemachine
@states([PREPARING, IN_TRANSIT, OUT_FOR_DELIVERY, DELIVERED])
@scene(domain: "logistics")
entity Shipment {
    @unique
    tracking_number: String
    
    carrier: String
    
    @min(0)
    weight: Decimal
    
    shipped_at: DateTime
    delivered_at: DateTime
    
    @transition(event: "dispatch", from: PREPARING, to: IN_TRANSIT)
    @transition(event: "outForDelivery", from: IN_TRANSIT, to: OUT_FOR_DELIVERY)
    @transition(event: "deliver", from: OUT_FOR_DELIVERY, to: DELIVERED)
    status: String
    
    relation order -> Order
}

// ==================== 内容管理 ====================

@extends(AuditableEntity)
@statemachine
@states([DRAFT, UNDER_REVIEW, APPROVED, PUBLISHED, ARCHIVED])
@scene(domain: "content_management")
@position(T: "2024", S: "editorial", Fa: "content", Fu: "publishing")
@permission(admin, [create, read, update, delete])
@permission(editor, [create, read, update, publish])
@permission(author, [create, read, update_own])
entity Post {
    @unique
    slug: String
    
    @minLength(5) @maxLength(200)
    title: String
    
    @quality(completeness, 0.90)
    content: Text
    
    @quality(timeliness, 0.85)
    published_at: DateTime
    
    @readRole([all])
    view_count: Integer
    
    @onTransition(from: DRAFT, to: UNDER_REVIEW)
    fn notifyReviewers() { }
    
    @onTransition(from: APPROVED, to: PUBLISHED)
    fn publishPost() { }
    
    @transition(event: "submit", from: DRAFT, to: UNDER_REVIEW, guard: "isComplete")
    @transition(event: "approve", from: UNDER_REVIEW, to: APPROVED)
    @transition(event: "publish", from: APPROVED, to: PUBLISHED)
    @transition(event: "archive", from: PUBLISHED, to: ARCHIVED)
    status: String
    
    relation author -> User
    relation category -> PostCategory
    relation tags -> Tag[*]
}

@extends(BaseEntity)
entity PostCategory {
    @unique
    slug: String
    name: String
    ?description: Text
    
    relation posts -> Post[*]
}

@extends(BaseEntity)
entity Tag {
    @unique
    slug: String
    name: String
    
    relation posts -> Post[*]
}

// ==================== 规则与约束 ====================

@rule(name: "globalInventoryCheck", condition: "Product.stock_quantity >= 0", action: "validate")
@rule(name: "orderTotalCheck", condition: "Order.total_amount == Order.subtotal + Order.tax_amount + Order.shipping_amount")
entity GlobalRules {
    description: String
}

// ==================== 异常处理器 ====================

@handler(ValidationError)
@httpStatus(400)
@priority(1)
fn handleValidationError() {
    // Log validation errors
    // Return formatted error response
}

@handler(NotFoundError)
@httpStatus(404)
@priority(2)
fn handleNotFoundError() {
    // Log not found errors
    // Return 404 response
}

@handler(BusinessException)
@httpStatus(500)
@priority(99)
fn handleBusinessException() {
    // Log business exceptions
    // Return error response
}

// ==================== 示例多维度查询视图 ====================

// 按时间和领域分析订单
@view(type: "analytics")
@dimension(T: quarter, S: domain, Fa: product_category)
entity OrderAnalytics {
    period: String
    domain: String
    category: String
    total_orders: Integer
    total_revenue: Decimal
    average_order_value: Decimal
}

// 按职能和场景的用户活动
@view(type: "activity")
@dimension(Fu: role, S: context)
entity UserActivity {
    role: String
    context: String
    active_users: Integer
    actions_count: Integer
}
