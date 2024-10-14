CREATE TABLE `products`
(
    `id`          bigint unsigned                         NOT NULL AUTO_INCREMENT,
    `name`        varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
    `description` varchar(255) COLLATE utf8mb4_unicode_ci          DEFAULT NULL,
    `price`       decimal(8, 2)                           NOT NULL,
    `code`        int                                              DEFAULT NULL,
    `stock`       int                                     NOT NULL DEFAULT '0',
    `category_id` bigint unsigned                         NOT NULL,
    `image`       varchar(255) COLLATE utf8mb4_unicode_ci          DEFAULT NULL,
    `created_at`  timestamp                               NULL     DEFAULT NULL,
    `updated_at`  timestamp                               NULL     DEFAULT NULL,
    PRIMARY KEY (`id`),
) ENGINE = InnoDB
  AUTO_INCREMENT = 101
  DEFAULT CHARSET = utf8mb4
  COLLATE = utf8mb4_unicode_ci;
