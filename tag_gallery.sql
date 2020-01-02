-- Adminer 4.7.5 MySQL dump

SET NAMES utf8;
SET time_zone = '+00:00';

SET NAMES utf8mb4;

CREATE DATABASE `tag_gallery` /*!40100 DEFAULT CHARACTER SET utf8mb4 */;
USE `tag_gallery`;

DROP TABLE IF EXISTS `photos`;
CREATE TABLE `photos` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `source` int(11) NOT NULL,
  `relative_path` varchar(128) NOT NULL,
  `filesize` int(11) NOT NULL,
  `exif_latitude` double NOT NULL,
  `exif_longitude` double NOT NULL,
  `exif_altitude` double NOT NULL,
  `exif_gps_date` date NOT NULL,
  `exif_gps_time` time NOT NULL,
  `exif_datetime` time NOT NULL,
  `exif_width` int(11) NOT NULL,
  `exif_height` int(11) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `source_relative_path` (`source`,`relative_path`),
  CONSTRAINT `photos_ibfk_1` FOREIGN KEY (`source`) REFERENCES `sources` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


DROP TABLE IF EXISTS `photos_tags`;
CREATE TABLE `photos_tags` (
  `photo_id` int(11) NOT NULL AUTO_INCREMENT,
  `tag` varchar(128) NOT NULL,
  `type` enum('manual','exif') NOT NULL,
  `value` text NOT NULL,
  `creation_date` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`photo_id`),
  KEY `tag` (`tag`),
  CONSTRAINT `photos_tags_ibfk_1` FOREIGN KEY (`photo_id`) REFERENCES `photos` (`id`),
  CONSTRAINT `photos_tags_ibfk_2` FOREIGN KEY (`tag`) REFERENCES `tags` (`tag`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


DROP TABLE IF EXISTS `sources`;
CREATE TABLE `sources` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `full_path` varchar(255) NOT NULL,
  `status` enum('new','resized','indexed') NOT NULL DEFAULT 'new',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


DROP TABLE IF EXISTS `tags`;
CREATE TABLE `tags` (
  `tag` varchar(128) NOT NULL,
  `creation_date` timestamp NOT NULL DEFAULT '0000-00-00 00:00:00' ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`tag`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;


-- 2020-01-02 19:40:47
