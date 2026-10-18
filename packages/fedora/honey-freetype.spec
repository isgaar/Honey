Name:           honey-freetype
Version:        0.7.0
Release:        1%{?dist}
Summary:        Quartz-parity typography engine in Rust replacing libfreetype.so.6

License:        MIT
URL:            https://github.com/isgaar/Honey
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  gcc

Provides:       libfreetype.so.6()(64bit)
Provides:       freetype = 2.13.2

%description
Honey is a native OS typography engine implemented in pure Rust, designed
to replace FreeType with complete Apple Quartz Compositor parity. It features
analytic fractional rasterization, calibrated optical stem dilation, and
photometric linear gamma blending.

%prep
%autosetup -n Honey-%{version}

%build
cargo build --release -p honey-shim

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_libdir}/honey
mkdir -p %{buildroot}%{_sysconfdir}/ld.so.conf.d

install -m 755 target/release/libhoney_shim.so %{buildroot}%{_libdir}/honey/libfreetype.so.6
echo "%{_libdir}/honey" > %{buildroot}%{_sysconfdir}/ld.so.conf.d/00-honey.conf

%post
/sbin/ldconfig

%postun
/sbin/ldconfig

%files
%license LICENSE
%{_libdir}/honey/libfreetype.so.6
%config(noreplace) %{_sysconfdir}/ld.so.conf.d/00-honey.conf
