include $(TOPDIR)/rules.mk

PKG_NAME:=duinodcx-management
PKG_VERSION:=0.1.0
PKG_RELEASE:=1

PKG_SOURCE_PROTO:=git
PKG_SOURCE_URL:=https://github.com/ChongZhiJie/WinRS-DCX.git
PKG_SOURCE_DATE:=2026-05-14
PKG_SOURCE_VERSION:=main

PKG_MAINTAINER:=ChongZhiJie
PKG_LICENSE:=MIT

PKG_BUILD_DEPENDS:=rust/host

include $(INCLUDE_DIR)/package.mk
include $(INCLUDE_DIR)/nls.mk
include ../../lang/rust/rust-package.mk

define Package/duinodcx-management
  SECTION:=utils
  CATEGORY:=Utilities
  TITLE:=WinRS-DCX Management for DCX2496
  DEPENDS:=$(RUST_ARCH_DEPENDS)
  URL:=https://github.com/ChongZhiJie/WinRS-DCX
endef

define Package/duinodcx-management/description
  WinRS-DCX is a management application for the Behringer DCX2496 audio processor.
endef

define Package/duinodcx-management/install
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_INSTALL_DIR)/bin/WinRS-DCX-Management $(1)/usr/bin/duinodcx
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/duinodcx.init $(1)/etc/init.d/duinodcx
endef

$(eval $(call BuildPackage,duinodcx-management))
